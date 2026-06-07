#![doc = include_str!("../README.md")]
#![cfg_attr(not(feature = "std"), no_std)]

use core::{
    borrow::Borrow,
    cmp::PartialOrd,
    convert::Infallible,
    error::Error,
    fmt,
    hash::{Hash, Hasher},
    marker::PhantomData,
};

pub mod aliases;

#[cfg(feature = "serde")]
mod serde;

/// Validation strategy for [`GType`].
///
/// A validator can:
///
/// - Define an optional minimum value via [`Validator::min`].
/// - Define an optional maximum value via [`Validator::max`].
/// - Perform arbitrary validation via [`Validator::validate`].
///
/// Validation is performed when constructing a [`GType`] using
/// [`GType::try_new`].
///
/// # Example
///
/// ```rust
/// use core::convert::Infallible;
/// use g_type::{GType, Validator};
///
/// struct Percent;
///
/// impl Validator<u8> for Percent {
///     type Target = u8;
///     type Error = Infallible;
///
///     fn max() -> Option<&'static Self::Target> {
///         Some(&100)
///     }
/// }
///
/// let value = GType::<u8, Percent>::try_new(42);
/// assert!(value.is_ok());
/// ```
pub trait Validator<T> {
    /// Target type for bounds comparison.
    ///
    /// # Examples
    /// - u32 -> u32
    /// - String -> str
    /// - Vec\<T\> -> \[T\] or \[T; N\]
    type Target: PartialOrd<T> + ?Sized + 'static;

    /// Validation error type.
    type Error;

    /// Minimum value in range, inclusive.
    #[inline]
    fn min() -> Option<&'static Self::Target> {
        None
    }

    /// Maximum value in range, inclusive.
    #[inline]
    fn max() -> Option<&'static Self::Target> {
        None
    }

    /// Validation logics.
    #[inline]
    fn validate(_: &T) -> Result<(), Self::Error> {
        Ok(())
    }
}

/// Validator that performs no validation.
///
/// This is the default validator used by [`GType`].
///
/// ```rust
/// use g_type::GType;
///
/// let value = GType::<u32>::try_new(123).unwrap();
/// ```
#[derive(Debug, Clone, Copy)]
pub struct NoValidation;

impl<T: PartialOrd + 'static> Validator<T> for NoValidation {
    type Target = T;
    type Error = Infallible;
}

impl<T: PartialOrd + 'static> Validator<T> for () {
    type Target = T;
    type Error = Infallible;
}

/// Error returned when constructing a [`GType`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GTypeError<E> {
    /// The value is below the validator's minimum bound.
    BelowMinimum,

    /// The value is above the validator's maximum bound.
    AboveMaximum,

    /// The validator rejected the value.
    Validation(E),
}

impl<E: fmt::Display> fmt::Display for GTypeError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BelowMinimum => f.write_str("value is below minimum"),
            Self::AboveMaximum => f.write_str("value is above maximum"),
            Self::Validation(err) => err.fmt(f),
        }
    }
}

impl<E: Error + 'static> Error for GTypeError<E> {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Validation(err) => Some(err),
            Self::BelowMinimum | Self::AboveMaximum => None,
        }
    }
}

/// A validated value.
///
/// `GType<T, V>` wraps a value of type `T` and guarantees that it
/// satisfied validator `V` at construction time.
///
/// Validation is performed by [`GType::try_new`]. Once constructed,
/// the contained value cannot be modified directly, preserving the
/// validator's invariants.
///
/// # Examples
///
/// ```rust
/// use g_type::GType;
///
/// let value = GType::<u32>::try_new(42).unwrap();
///
/// assert_eq!(*value.as_ref(), 42);
/// ```
///
/// With a custom validator:
///
/// ```rust
/// use core::convert::Infallible;
/// use g_type::{GType, Validator};
///
/// struct Percent;
///
/// impl Validator<u8> for Percent {
///     type Target = u8;
///     type Error = Infallible;
///
///     fn max() -> Option<&'static Self::Target> {
///         Some(&100)
///     }
/// }
///
/// assert!(GType::<u8, Percent>::try_new(50).is_ok());
/// assert!(GType::<u8, Percent>::try_new(150).is_err());
/// ```
#[repr(transparent)]
pub struct GType<T, V = NoValidation> {
    value: T,
    _marker: PhantomData<V>,
}

impl<T: PartialOrd<V::Target>, V: Validator<T>> GType<T, V> {
    #[inline]
    pub(crate) const fn new_unchecked(value: T) -> Self {
        Self {
            value,
            _marker: PhantomData,
        }
    }

    /// Attempts to create a validated value.
    ///
    /// Returns an error if the value violates the validator's
    /// minimum bound, maximum bound, or custom validation rules.
    pub fn try_new(value: T) -> Result<Self, GTypeError<V::Error>> {
        if let Some(min) = V::min()
            && &value < min
        {
            return Err(GTypeError::BelowMinimum);
        }
        if let Some(max) = V::max()
            && &value > max
        {
            return Err(GTypeError::AboveMaximum);
        }

        V::validate(&value).map_err(GTypeError::Validation)?;

        Ok(Self::new_unchecked(value))
    }

    /// Returns a shared reference to the underlying value.
    #[inline]
    pub const fn as_ref(&self) -> &T {
        &self.value
    }

    /// Consumes the wrapper and returns the underlying value.
    #[inline]
    pub fn into_inner(self) -> T {
        self.value
    }

    /// Calls a function with a reference to the contained value.
    ///
    /// Returns `self` unchanged.
    ///
    /// This is primarily useful for debugging and logging in method
    /// chains.
    #[inline]
    pub fn inspect<F>(self, func: F) -> Self
    where
        F: FnOnce(&T),
    {
        func(&self.value);
        self
    }

    /// Transforms the contained value into another validated type.
    ///
    /// The transformed value is validated using the destination
    /// validator before being returned.
    ///
    /// This behaves similarly to `Option::map` and `Result::map`.
    #[inline]
    pub fn map<U, UV, F>(self, func: F) -> Result<GType<U, UV>, GTypeError<UV::Error>>
    where
        U: PartialOrd<UV::Target>,
        UV: Validator<U>,
        F: FnOnce(T) -> U,
    {
        GType::<U, UV>::try_new(func(self.value))
    }

    /// Chains another fallible validated transformation.
    ///
    /// This behaves similarly to `Option::and_then` and
    /// `Result::and_then`.
    #[inline]
    pub fn and_then<U, UV, F>(self, func: F) -> Result<GType<U, UV>, GTypeError<UV::Error>>
    where
        U: PartialOrd<UV::Target>,
        UV: Validator<U>,
        F: FnOnce(T) -> Result<GType<U, UV>, GTypeError<UV::Error>>,
    {
        func(self.value)
    }
}

impl<T, V> AsRef<T> for GType<T, V> {
    #[inline]
    fn as_ref(&self) -> &T {
        &self.value
    }
}

impl<T, V> Borrow<T> for GType<T, V> {
    #[inline]
    fn borrow(&self) -> &T {
        &self.value
    }
}

impl<T, V> Clone for GType<T, V>
where
    T: Clone,
{
    #[inline]
    fn clone(&self) -> Self {
        Self {
            value: self.value.clone(),
            _marker: PhantomData,
        }
    }
}

impl<T, V> Copy for GType<T, V> where T: Copy {}

impl<T, V> fmt::Debug for GType<T, V>
where
    T: fmt::Debug,
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.value.fmt(f)
    }
}

impl<T, V> fmt::Display for GType<T, V>
where
    T: fmt::Display,
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.value.fmt(f)
    }
}

impl<T, U, LHSV, RHSV> PartialEq<GType<U, RHSV>> for GType<T, LHSV>
where
    T: PartialEq<U>,
    U: PartialEq<T>,
{
    #[inline]
    fn eq(&self, other: &GType<U, RHSV>) -> bool {
        self.value == other.value
    }
}

impl<T, V> Eq for GType<T, V> where T: Eq {}

impl<T, U, LHSV, RHSV> PartialOrd<GType<U, RHSV>> for GType<T, LHSV>
where
    T: PartialOrd<U>,
    U: PartialOrd<T>,
{
    #[inline]
    fn partial_cmp(&self, other: &GType<U, RHSV>) -> Option<core::cmp::Ordering> {
        self.value.partial_cmp(&other.value)
    }
}

impl<T, V> Ord for GType<T, V>
where
    T: Ord,
{
    #[inline]
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.value.cmp(&other.value)
    }
}

impl<T, V> Hash for GType<T, V>
where
    T: Hash,
{
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.value.hash(state);
    }
}
