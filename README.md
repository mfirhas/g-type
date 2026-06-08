# GType

![Rust](https://img.shields.io/badge/Rust-000000?style=flat&logo=rust&logoColor=white)
[![Crates.io](https://img.shields.io/crates/v/g-type.svg)](https://crates.io/crates/g-type)
[![ci](https://github.com/mfirhas/g-type/actions/workflows/ci.yml/badge.svg)](https://github.com/mfirhas/g-type/actions/workflows/ci.yml)
[![Documentation](https://docs.rs/g-type/badge.svg)](https://docs.rs/g-type)
[![codecov](https://codecov.io/gh/mfirhas/g-type/branch/master/graph/badge.svg)](https://codecov.io/gh/mfirhas/g-type)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://github.com/mfirhas/g-type/blob/master/LICENSE)

A lightweight, `no_std`-friendly validated value type.

`GType<T, V>` wraps a value of type `T` and guarantees that it satisfies the constraints defined by a validator `V`.

Validators can provide:

* Minimum and maximum bounds
* Arbitrary validation logic
* Custom error types

## Features

* `no_std` compatible
* Zero-cost abstraction after construction
* Optional validation
* Compile-time validator definitions
* Custom validation errors
* Works with primitive and user-defined types
* Preserves common traits such as `Clone`, `Copy`, `Eq`, `Ord`, `Hash`, `Display`, and `Debug`

## Basic Usage

```rust
use g_type::{GType, Validator};

struct Percent;

impl Validator<u8> for Percent {
    type Target = u8;
    type Error = core::convert::Infallible;

    fn min() -> Option<&'static Self::Target> {
        Some(&0)
    }

    fn max() -> Option<&'static Self::Target> {
        Some(&100)
    }
}

type Percentage = GType<u8, Percent>;

let value = Percentage::try_new(75).unwrap();
assert_eq!(value.into_inner(), 75);
```

## Custom Validation

Validators may perform arbitrary runtime checks.

```rust
use g_type::{GType, Validator};

#[derive(Debug, Clone, PartialEq, Eq)]
struct EvenError;

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

type EvenNumber = GType<u32, Even>;

assert!(EvenNumber::try_new(4).is_ok());
assert!(EvenNumber::try_new(5).is_err());
```

## No Validation

Use the default validator when no validation is required.

```rust
use g_type::GType;

let value = g_type::GType::<u32>::try_new(42).unwrap();
assert_eq!(value.into_inner(), 42);
```

## Transforming Values

`map()` transforms the inner value and validates the result using the destination validator.

```rust
let value = g_type::GType::<u32>::try_new(50).unwrap();

struct Percent;

impl g_type::Validator<u8> for Percent {
    type Target = u8;
    type Error = core::convert::Infallible;

    fn min() -> Option<&'static Self::Target> {
        Some(&0)
    }

    fn max() -> Option<&'static Self::Target> {
        Some(&100)
    }
}

let percent = value.map::<u8, Percent, _>(|v| v as u8).unwrap();

assert_eq!(percent.into_inner(), 50);
```

`and_then()` allows chaining validated transformations.

```rust
let value = g_type::GType::<u32>::try_new(50).unwrap();

struct Percent;

impl g_type::Validator<u8> for Percent {
    type Target = u8;
    type Error = core::convert::Infallible;

    fn min() -> Option<&'static Self::Target> {
        Some(&0)
    }

    fn max() -> Option<&'static Self::Target> {
        Some(&100)
    }
}

let percent = value.and_then::<u8, Percent, _>(|v| {
    g_type::GType::<u8, Percent>::try_new(v as u8)
}).unwrap();
```

## Error Handling

Construction may fail for three reasons:

```rust
pub enum GTypeError<E> {
    MinExceedsMax,
    BelowMinimum,
    AboveMaximum,
    Validation(E),
}
```

* `MinExceedsMax` — minimum value exceeds maximum value. 
* `BelowMinimum` — value is below the validator minimum.
* `AboveMaximum` — value exceeds the validator maximum.
* `Validation(E)` — custom validator rejected the value.

## Design

A validator is a type implementing:

```rust
pub trait Validator<T> {
    /// Target type for bounds comparison.
    ///
    /// # Examples
    /// - u32 -> u32
    /// - String -> str
    /// - Vec\<T\> -> \[T\] or \[T; N\]
    type Target: PartialOrd<Self::Target> + PartialOrd<T> + ?Sized + 'static;

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
```

Validators are stateless marker types. All constraints are defined through associated functions, making them easy to use in `const` contexts and zero-sized at runtime.
