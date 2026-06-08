use std::convert::Infallible;

use g_type::aliases::*;
use g_type::*;
use serde_json;

pub struct Percent;

impl Validator<u8> for Percent {
    type Target = u8;
    type Error = Infallible;

    fn min() -> Option<&'static Self::Target> {
        Some(&0)
    }

    fn max() -> Option<&'static Self::Target> {
        Some(&100)
    }
}

#[test]
fn serialize_valid_value() {
    let value = Gu8::<Percent>::try_new(42).unwrap();

    let json = serde_json::to_string(&value).unwrap();

    assert_eq!(json, "42");
}

#[test]
fn deserialize_valid_value() {
    let value: Gu8<Percent> = serde_json::from_str("42").unwrap();

    assert_eq!(value.into_inner(), 42);
}

#[test]
fn deserialize_below_minimum() {
    #[derive(Debug)]
    struct Positive;

    impl Validator<i32> for Positive {
        type Target = i32;
        type Error = core::convert::Infallible;

        fn min() -> Option<&'static Self::Target> {
            static MIN: i32 = 1;
            Some(&MIN)
        }
    }

    let result = serde_json::from_str::<Gi32<Positive>>("0");

    assert!(result.is_err());
}

#[test]
fn deserialize_above_maximum() {
    let result = serde_json::from_str::<Gu8<Percent>>("101");

    assert!(result.is_err());
}

#[test]
fn roundtrip() {
    let original = Gu8::<Percent>::try_new(75).unwrap();

    let json = serde_json::to_string(&original).unwrap();

    let decoded: Gu8<Percent> = serde_json::from_str(&json).unwrap();

    assert_eq!(original, decoded);
}

#[test]
fn deserialize_preserves_validation() {
    let ok: Result<Gu8<Percent>, _> = serde_json::from_str("100");

    let err: Result<Gu8<Percent>, _> = serde_json::from_str("255");

    assert!(ok.is_ok());
    assert!(err.is_err());
}

#[test]
fn failed_deserialize() {
    let err: Result<Gu8<Percent>, _> = serde_json::from_str("anu");

    dbg!(&err);
    assert!(err.is_err());
}

#[test]
fn failed_deserialize_invalid() {
    let err: Result<Gu8<Percent>, _> = serde_json::from_str("200");

    dbg!(&err);
    assert!(err.is_err());
}
