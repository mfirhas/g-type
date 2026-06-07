use core::convert::Infallible;
use g_type::*;
use std::borrow::Borrow;
use std::collections::hash_map::DefaultHasher;
use std::error::Error;
use std::hash::{Hash, Hasher};

// ===== Validators ========================================================

struct Percent;

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

#[derive(Debug, Clone, PartialEq, Eq)]
struct EvenError;

impl core::fmt::Display for EvenError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("must be even")
    }
}

impl Error for EvenError {}

struct Even;

impl Validator<u32> for Even {
    type Target = u32;
    type Error = EvenError;

    fn validate(value: &u32) -> Result<(), Self::Error> {
        if value % 2 == 0 {
            Ok(())
        } else {
            Err(EvenError)
        }
    }
}

// ===== try_new ===========================================================

#[test]
fn try_new_percent_table() {
    struct Case {
        value: u8,
        expected: Result<u8, GTypeError<Infallible>>,
    }

    let cases = [
        Case {
            value: 0,
            expected: Ok(0),
        },
        Case {
            value: 50,
            expected: Ok(50),
        },
        Case {
            value: 100,
            expected: Ok(100),
        },
        Case {
            value: 101,
            expected: Err(GTypeError::AboveMaximum),
        },
    ];

    for case in cases {
        let actual = GType::<u8, Percent>::try_new(case.value).map(|v| v.into_inner());

        assert_eq!(actual, case.expected);
    }
}

#[test]
fn try_new_validation_table() {
    struct Case {
        value: u32,
        expected: Result<u32, GTypeError<EvenError>>,
    }

    let cases = [
        Case {
            value: 2,
            expected: Ok(2),
        },
        Case {
            value: 4,
            expected: Ok(4),
        },
        Case {
            value: 7,
            expected: Err(GTypeError::Validation(EvenError)),
        },
    ];

    for case in cases {
        let actual = GType::<u32, Even>::try_new(case.value).map(|v| v.into_inner());

        assert_eq!(actual, case.expected);
    }
}

// ===== inspect ===========================================================

#[test]
fn inspect_calls_closure_and_returns_self() {
    let mut seen = None;

    let value = GType::<u32>::try_new(123)
        .unwrap()
        .inspect(|v| seen = Some(*v));

    assert_eq!(seen, Some(123));
    assert_eq!(value.into_inner(), 123);
}

// ===== map ===============================================================

#[test]
fn map_table() {
    struct Case {
        input: u32,
        expected: Result<u8, GTypeError<Infallible>>,
    }

    let cases = [
        Case {
            input: 0,
            expected: Ok(0),
        },
        Case {
            input: 50,
            expected: Ok(50),
        },
        Case {
            input: 100,
            expected: Ok(100),
        },
        Case {
            input: 101,
            expected: Err(GTypeError::AboveMaximum),
        },
    ];

    for case in cases {
        let value = GType::<u32>::try_new(case.input).unwrap();

        let actual = value
            .map::<u8, Percent, _>(|v| v as u8)
            .map(|v| v.into_inner());

        assert_eq!(actual, case.expected);
    }
}

#[test]
fn map_non_copy_string() {
    let value = GType::<String>::try_new("hello".to_owned()).unwrap();

    let mapped = value.map::<usize, NoValidation, _>(|s| s.len()).unwrap();

    assert_eq!(mapped.into_inner(), 5);
}

// ===== and_then ==========================================================

#[test]
fn and_then_success() {
    let value = GType::<u32>::try_new(42).unwrap();

    let result = value
        .and_then::<u8, Percent, _>(|v| GType::<u8, Percent>::try_new(v as u8))
        .unwrap();

    assert_eq!(result.into_inner(), 42);
}

#[test]
fn and_then_error() {
    let value = GType::<u32>::try_new(150).unwrap();

    let result = value.and_then::<u8, Percent, _>(|v| GType::<u8, Percent>::try_new(v as u8));

    assert_eq!(result, Err(GTypeError::AboveMaximum));
}

// ===== traits ============================================================

#[test]
fn clone_works_for_non_copy_type() {
    let value = GType::<String>::try_new("hello".to_owned()).unwrap();

    let cloned = value.clone();

    assert_eq!(value, cloned);
}

#[test]
fn copy_works_for_copy_type() {
    let a = GType::<u32>::try_new(123).unwrap();
    let b = a;

    assert_eq!(a, b);
}

#[test]
fn display_and_debug_delegate() {
    let value = GType::<u32>::try_new(123).unwrap();

    assert_eq!(format!("{value}"), "123");
    assert_eq!(format!("{value:?}"), "123");
}

#[test]
fn ordering_delegates_to_inner() {
    let a = GType::<u32>::try_new(1).unwrap();
    let b = GType::<u32>::try_new(2).unwrap();

    assert!(a < b);
    assert!(b > a);
}

#[test]
fn hash_matches_inner_value() {
    let value = GType::<u32>::try_new(123).unwrap();

    let mut h1 = DefaultHasher::new();
    value.hash(&mut h1);

    let mut h2 = DefaultHasher::new();
    123u32.hash(&mut h2);

    assert_eq!(h1.finish(), h2.finish());
}

#[test]
fn as_ref_and_borrow_delegate() {
    let value = GType::<String>::try_new("hello".to_owned()).unwrap();

    let bor: &String = value.borrow();
    assert_eq!(value.as_ref(), "hello");
    assert_eq!(bor, "hello");
}

// ===== error =============================================================

#[test]
fn error_display_messages() {
    assert_eq!(
        GTypeError::<Infallible>::BelowMinimum.to_string(),
        "value is below minimum"
    );

    assert_eq!(
        GTypeError::<Infallible>::AboveMaximum.to_string(),
        "value is above maximum"
    );

    assert_eq!(
        GTypeError::<EvenError>::Validation(EvenError).to_string(),
        "must be even"
    );
}

#[test]
fn validation_error_exposes_source() {
    let err = GTypeError::Validation(EvenError);

    assert!(err.source().is_some());
}
