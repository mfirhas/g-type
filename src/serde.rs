use core::fmt::Debug;

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
    V::Target: PartialOrd<T> + Debug,
    V::Error: core::fmt::Display,
{
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = T::deserialize(deserializer).map_err(|err| {
            D::Error::custom(format_args!(
                "failed deserializin T({}): {}",
                core::any::type_name::<T>(),
                err
            ))
        })?;

        GType::<T, V>::try_new(value).map_err(|err| {
            D::Error::custom(format_args!(
                "failed constructing {}, min:{:?}, max:{:?}: {}",
                core::any::type_name::<Self>(),
                V::min(),
                V::max(),
                err
            ))
        })
    }
}
