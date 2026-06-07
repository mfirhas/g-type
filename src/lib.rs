#![no_std]

use core::{
    borrow::Borrow,
    cmp::PartialOrd,
    convert::Infallible,
    fmt,
    hash::{Hash, Hasher},
    marker::PhantomData,
};

/// Optional runtime validation.
pub trait Validator<T> {
    type Target: PartialOrd<T> + ?Sized + 'static;
    type Error;

    #[inline]
    fn min() -> Option<&'static Self::Target> {
        None
    }

    #[inline]
    fn max() -> Option<&'static Self::Target> {
        None
    }

    #[inline]
    fn validate(_: &T) -> Result<(), Self::Error> {
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NoValidation;

impl<T: PartialOrd + 'static> Validator<T> for NoValidation {
    type Target = T;
    type Error = Infallible;
}

impl<T: PartialOrd + 'static> Validator<T> for () {
    type Target = T;
    type Error = Infallible;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GTypeError<E> {
    BelowMinimum,
    AboveMaximum,
    Validation(E),
}

#[repr(transparent)]
pub struct GType<T, V = NoValidation> {
    value: T,
    _marker: PhantomData<V>,
}

impl<T, V: Validator<T>> GType<T, V>
where
    T: PartialOrd<V::Target>,
{
    #[inline]
    pub(crate) const fn new_unchecked(value: T) -> Self {
        Self {
            value,
            _marker: PhantomData,
        }
    }

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

    #[inline]
    pub const fn as_ref(&self) -> &T {
        &self.value
    }

    #[inline]
    pub fn into_inner(self) -> T {
        self.value
    }

    #[inline]
    pub fn inspect<F>(self, func: F) -> Self
    where
        F: FnOnce(&T),
    {
        func(&self.value);
        self
    }

    #[inline]
    pub fn map<U, UV, F>(self, func: F) -> Result<GType<U, UV>, GTypeError<UV::Error>>
    where
        F: FnOnce(T) -> U,
        U: PartialOrd<UV::Target>,
        UV: Validator<U>,
    {
        GType::<U, UV>::try_new(func(self.value))
    }

    #[inline]
    pub fn and_then<F>(self, func: F) -> Result<Self, GTypeError<V::Error>>
    where
        F: FnOnce(T) -> Result<Self, GTypeError<V::Error>>,
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

impl<T, V> PartialEq for GType<T, V>
where
    T: PartialEq,
{
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.value.eq(&other.value)
    }
}

impl<T, V> Eq for GType<T, V> where T: Eq {}

impl<T, V> PartialOrd for GType<T, V>
where
    T: PartialOrd,
{
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
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
