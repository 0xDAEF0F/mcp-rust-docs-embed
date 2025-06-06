## [tap](../../tap/index.html)1.0.1

## Module conv

### Sections

* [Method-Directed Type Conversion](#method-directed-type-conversion "Method-Directed Type Conversion")

### [Module Items](#traits)

* [Traits](#traits "Traits")

## [In crate tap](../index.html)

[tap](../index.html)

# Module convCopy item path

[Source](../../src/tap/conv.rs.html#1-87)

Expand description

## [§](#method-directed-type-conversion)Method-Directed Type Conversion

The `std::convert` module provides traits for converting values from one type to
another. The first of these, [`From<T>`](https://doc.rust-lang.org/std/convert/trait.From.html), provides an associated function
[`from(orig: T) -> Self`](https://doc.rust-lang.org/std/convert/trait.From.html#tymethod.from). This function can only be called in prefix-position,
as it does not have a `self` receiver. The second, [`Into<T>`](https://doc.rust-lang.org/std/convert/trait.Into.html), provides a
method [`into(self) -> T`](https://doc.rust-lang.org/std/convert/trait.Into.html#tymethod.into) which *can* be called in suffix-position; due to
intractable problems in the type solver, this method cannot have any *further*
method calls attached to it. It must be bound directly into a `let` or function
call.

The [`TryFrom<T>`](https://doc.rust-lang.org/std/convert/trait.TryFrom.html) and [`TryInto<T>`](https://doc.rust-lang.org/std/convert/trait.TryInto.html) traits have the same properties, but
permit failure.

This module provides traits that place the conversion type parameter in the
method, rather than in the trait, so that users can write `.conv::<T>()` to
convert the preceding expression into `T`, without causing any failures in the
type solver. These traits are blanket-implemented on all types that have an
`Into<T>` implementation, which covers both the blanket implementation of `Into`
for types with `From`, and manual implementations of `Into`.

!

## Traits[§](#traits)

[Conv](trait.Conv.html "trait tap::conv::Conv")
:   Wraps `Into::<T>::into` as a method that can be placed in pipelines.

[TryConv](trait.TryConv.html "trait tap::conv::TryConv")
:   Wraps `TryInto::<T>::try_into` as a method that can be placed in pipelines.