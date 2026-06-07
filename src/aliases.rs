//! Some aliases for some primitives.
//!

use crate::{GType, NoValidation};

pub type Gu8<V = NoValidation> = GType<u8, V>;
pub type Gu16<V = NoValidation> = GType<u16, V>;
pub type Gu32<V = NoValidation> = GType<u32, V>;
pub type Gu64<V = NoValidation> = GType<u64, V>;
pub type Gu128<V = NoValidation> = GType<u128, V>;
pub type Gusize<V = NoValidation> = GType<usize, V>;

pub type Gi8<V = NoValidation> = GType<i8, V>;
pub type Gi16<V = NoValidation> = GType<i16, V>;
pub type Gi32<V = NoValidation> = GType<i32, V>;
pub type Gi64<V = NoValidation> = GType<i64, V>;
pub type Gi128<V = NoValidation> = GType<i128, V>;
pub type Gisize<V = NoValidation> = GType<isize, V>;

pub type Gf32<V = NoValidation> = GType<f32, V>;
pub type Gf64<V = NoValidation> = GType<f64, V>;

pub type GChar<V = NoValidation> = GType<char, V>;

#[cfg(feature = "alloc")]
pub type GString<V = NoValidation> = GType<String, V>;

pub type GStr<V = NoValidation> = GType<&'static str, V>;
