use crate::*;
use ::serde::{Deserialize, Deserializer, Serialize, Serializer, de::Error as DeError};

impl<T, V> Serialize for GType<T, V>
where
    T: Serialize,
{
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.value.serialize(serializer)
    }
}

impl<'de, T, V> Deserialize<'de> for GType<T, V>
where
    T: PartialOrd<V::Target> + Deserialize<'de>,
    V: Validator<T>,
    V::Target: PartialOrd<T>,
    V::Error: core::fmt::Display,
{
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = T::deserialize(deserializer)?;

        GType::<T, V>::try_new(value).map_err(D::Error::custom)
    }
}
