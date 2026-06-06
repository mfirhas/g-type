#![no_std]

use core::{
    borrow::Borrow,
    cmp::PartialOrd,
    convert::Infallible,
    fmt,
    hash::{Hash, Hasher},
    marker::PhantomData,
};

/// Compile-time bounds for const-capable types.
pub trait Range<T> {
    const MIN: T;
    const MAX: T;
}

/// Static borrowed bounds for non-const owned types.
///
/// Examples:
/// - String -> str
/// - Vec<u8> -> [u8]
/// - PathBuf -> Path
pub trait BorrowRange {
    type Borrowed: ?Sized + PartialOrd + 'static;

    fn min() -> &'static Self::Borrowed;
    fn max() -> &'static Self::Borrowed;
}

/// Optional runtime validation.
pub trait Validator<T> {
    type Error;

    fn validate(value: &T) -> Result<(), Self::Error>;
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NoValidation;

impl<T> Validator<T> for NoValidation {
    type Error = Infallible;

    #[inline]
    fn validate(_: &T) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl<T> Validator<T> for () {
    type Error = Infallible;

    #[inline]
    fn validate(_: &T) -> Result<(), Self::Error> {
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GTypeError<E> {
    BelowMinimum,
    AboveMaximum,
    Validation(E),
}

#[repr(transparent)]
pub struct GType<T, B, V = NoValidation> {
    value: T,
    _marker: PhantomData<(B, V)>,
}

impl<T, B, V> GType<T, B, V> {
    #[inline]
    const fn new_unchecked(value: T) -> Self {
        Self {
            value,
            _marker: PhantomData,
        }
    }

    #[inline]
    pub const fn get(&self) -> &T {
        &self.value
    }

    #[inline]
    pub fn into_inner(self) -> T {
        self.value
    }
}

/// Constructor for const-capable bounded types.
impl<T, B, V> GType<T, B, V>
where
    T: PartialOrd + Copy,
    B: Range<T>,
    V: Validator<T>,
{
    pub fn try_new(value: T) -> Result<Self, GTypeError<V::Error>> {
        if value < B::MIN {
            return Err(GTypeError::BelowMinimum);
        }

        if value > B::MAX {
            return Err(GTypeError::AboveMaximum);
        }

        V::validate(&value).map_err(GTypeError::Validation)?;

        Ok(Self::new_unchecked(value))
    }

    pub fn inspect<F>(self, func: F) -> Self
    where
        F: FnOnce(&T),
    {
        func(&self.value);
        self
    }

    #[inline]
    pub fn map<F, U, UB, UV>(&self, func: F) -> Result<GType<U, UB, UV>, GTypeError<UV::Error>>
    where
        F: FnOnce(&T) -> U,
        U: PartialOrd + Copy,
        UB: Range<U>,
        UV: Validator<U>,
    {
        GType::try_new(func(&self.value))
    }

    #[inline]
    pub fn and_then<F>(&self, func: F) -> Result<Self, GTypeError<V::Error>>
    where
        F: FnOnce(&T) -> Result<Self, GTypeError<V::Error>>,
    {
        func(&self.value)
    }
}

/// Constructor for borrowed/static bounded types.
impl<T, B, V> GType<T, B, V>
where
    B: BorrowRange,
    T: Borrow<B::Borrowed>,
    V: Validator<T>,
{
    pub fn try_owned(value: T) -> Result<Self, GTypeError<V::Error>> {
        let borrowed = value.borrow();

        if borrowed < B::min() {
            return Err(GTypeError::BelowMinimum);
        }

        if borrowed > B::max() {
            return Err(GTypeError::AboveMaximum);
        }

        V::validate(&value).map_err(GTypeError::Validation)?;

        Ok(Self::new_unchecked(value))
    }
}

impl<T, B, V> AsRef<T> for GType<T, B, V> {
    #[inline]
    fn as_ref(&self) -> &T {
        &self.value
    }
}

impl<T, B, V> Clone for GType<T, B, V>
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

impl<T, B, V> Copy for GType<T, B, V> where T: Copy {}

impl<T, B, V> fmt::Debug for GType<T, B, V>
where
    T: fmt::Debug,
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.value.fmt(f)
    }
}

impl<T, B, V> fmt::Display for GType<T, B, V>
where
    T: fmt::Display,
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.value.fmt(f)
    }
}

impl<T, B, V> PartialEq for GType<T, B, V>
where
    T: PartialEq,
{
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.value.eq(&other.value)
    }
}

impl<T, B, V> Eq for GType<T, B, V> where T: Eq {}

impl<T, B, V> PartialOrd for GType<T, B, V>
where
    T: PartialOrd,
{
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        self.value.partial_cmp(&other.value)
    }
}

impl<T, B, V> Ord for GType<T, B, V>
where
    T: Ord,
{
    #[inline]
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.value.cmp(&other.value)
    }
}

impl<T, B, V> Hash for GType<T, B, V>
where
    T: Hash,
{
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.value.hash(state);
    }
}
