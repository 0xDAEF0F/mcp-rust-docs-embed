## [tap](../../tap/index.html)1.0.1

## TapOptional

### [Required Associated Types](#required-associated-types)

* [Val](#associatedtype.Val "Val")

### [Required Methods](#required-methods)

* [tap\_none](#tymethod.tap_none "tap_none")
* [tap\_some](#tymethod.tap_some "tap_some")
* [tap\_some\_mut](#tymethod.tap_some_mut "tap_some_mut")

### [Provided Methods](#provided-methods)

* [tap\_none\_dbg](#method.tap_none_dbg "tap_none_dbg")
* [tap\_some\_dbg](#method.tap_some_dbg "tap_some_dbg")
* [tap\_some\_mut\_dbg](#method.tap_some_mut_dbg "tap_some_mut_dbg")

### [Implementations on Foreign Types](#foreign-impls)

* [Option<T>](#impl-TapOptional-for-Option%3CT%3E "Option<T>")

### [Dyn Compatibility](#dyn-compatibility)

### [Implementors](#implementors)

## [In tap::tap](index.html)

[tap](../index.html)::[tap](index.html)

# Trait TapOptionalCopy item path

[Source](../../src/tap/tap.rs.html#346-415)

```
pub trait TapOptional

where
    Self: Sized,

{
    type Val: ?Sized;

    // Required methods
    fn tap_some(self, func: impl FnOnce(&Self::Val)) -> Self;
    fn tap_some_mut(self, func: impl FnOnce(&mut Self::Val)) -> Self;
    fn tap_none(self, func: impl FnOnce()) -> Self;

    // Provided methods
    fn tap_some_dbg(self, func: impl FnOnce(&Self::Val)) -> Self { ... }
    fn tap_some_mut_dbg(self, func: impl FnOnce(&mut Self::Val)) -> Self { ... }
    fn tap_none_dbg(self, func: impl FnOnce()) -> Self { ... }
}
```

Expand description

Optional tapping, conditional on the optional presence of a value.

This trait is intended for use on types that express the concept of “optional
presence”, primarily the [`Option`](https://doc.rust-lang.org/std/option/enum.Option.html) monad. It provides taps that inspect the
container to determine if the effect function should execute or not.

> Note: This trait is a specialization of [`TapFallible`](trait.TapFallible.html), and exists because
> the [`std::ops::Try`](https://doc.rust-lang.org/std/ops/trait.Try.html) trait is still unstable. When `Try` stabilizes, this
> trait can be removed, and `TapFallible` blanket-applied to all `Try`
> implementors.

## Required Associated Types[§](#required-associated-types)

[Source](../../src/tap/tap.rs.html#351)

#### type [Val](#associatedtype.Val): ?[Sized](https://doc.rust-lang.org/nightly/core/marker/trait.Sized.html "trait core::marker::Sized")

The interior type that the container may or may not carry.

## Required Methods[§](#required-methods)

[Source](../../src/tap/tap.rs.html#361)

#### fn [tap\_some](#tymethod.tap_some)(self, func: impl [FnOnce](https://doc.rust-lang.org/nightly/core/ops/function/trait.FnOnce.html "trait core::ops::function::FnOnce")(&Self::[Val](../trait.TapOptional.html#associatedtype.Val "type tap::TapOptional::Val"))) -> Self

Immutabily accesses an interior value only when it is present.

This function is identical to [`Tap::tap`](trait.Tap.html#method.tap), except that it is required
to check the implementing container for value presence before running.
Implementors must not run the effect function if the container is marked
as being empty.

[Source](../../src/tap/tap.rs.html#371)

#### fn [tap\_some\_mut](#tymethod.tap_some_mut)(self, func: impl [FnOnce](https://doc.rust-lang.org/nightly/core/ops/function/trait.FnOnce.html "trait core::ops::function::FnOnce")(&mut Self::[Val](../trait.TapOptional.html#associatedtype.Val "type tap::TapOptional::Val"))) -> Self

Mutably accesses an interor value only when it is present.

This function is identical to [`Tap::tap_mut`](trait.Tap.html#method.tap_mut), except that it is
required to check the implementing container for value presence before
running. Implementors must not run the effect function if the container
is marked as being empty.

[Source](../../src/tap/tap.rs.html#381)

#### fn [tap\_none](#tymethod.tap_none)(self, func: impl [FnOnce](https://doc.rust-lang.org/nightly/core/ops/function/trait.FnOnce.html "trait core::ops::function::FnOnce")()) -> Self

Runs an effect function when the container is empty.

This function is identical to [`Tap::tap`](trait.Tap.html#method.tap), except that it is required
to check the implementing container for value absence before running.
Implementors must not run the effect function if the container is marked
as being non-empty.

## Provided Methods[§](#provided-methods)

[Source](../../src/tap/tap.rs.html#386-392)

#### fn [tap\_some\_dbg](#method.tap_some_dbg)(self, func: impl [FnOnce](https://doc.rust-lang.org/nightly/core/ops/function/trait.FnOnce.html "trait core::ops::function::FnOnce")(&Self::[Val](../trait.TapOptional.html#associatedtype.Val "type tap::TapOptional::Val"))) -> Self

Calls `.tap_some()` only in debug builds, and is erased in release
builds.

[Source](../../src/tap/tap.rs.html#397-403)

#### fn [tap\_some\_mut\_dbg](#method.tap_some_mut_dbg)(self, func: impl [FnOnce](https://doc.rust-lang.org/nightly/core/ops/function/trait.FnOnce.html "trait core::ops::function::FnOnce")(&mut Self::[Val](../trait.TapOptional.html#associatedtype.Val "type tap::TapOptional::Val"))) -> Self

Calls `.tap_some_mut()` only in debug builds, and is erased in release
builds.

[Source](../../src/tap/tap.rs.html#408-414)

#### fn [tap\_none\_dbg](#method.tap_none_dbg)(self, func: impl [FnOnce](https://doc.rust-lang.org/nightly/core/ops/function/trait.FnOnce.html "trait core::ops::function::FnOnce")()) -> Self

Calls `.tap_none()` only in debug builds, and is erased in release
builds.

## Dyn Compatibility[§](#dyn-compatibility)

This trait is **not** [dyn compatible](https://doc.rust-lang.org/nightly/reference/items/traits.html#dyn-compatibility).

*In older versions of Rust, dyn compatibility was called "object safety", so this trait is not object safe.*

## Implementations on Foreign Types[§](#foreign-impls)

[Source](../../src/tap/tap.rs.html#417-443)[§](#impl-TapOptional-for-Option%3CT%3E)

### impl<T> [TapOptional](../trait.TapOptional.html "trait tap::TapOptional") for [Option](https://doc.rust-lang.org/nightly/core/option/enum.Option.html "enum core::option::Option")<T>

[Source](../../src/tap/tap.rs.html#418)[§](#associatedtype.Val-1)

#### type [Val](#associatedtype.Val) = T

[Source](../../src/tap/tap.rs.html#421-426)[§](#method.tap_some)

#### fn [tap\_some](#tymethod.tap_some)(self, func: impl [FnOnce](https://doc.rust-lang.org/nightly/core/ops/function/trait.FnOnce.html "trait core::ops::function::FnOnce")([&T](https://doc.rust-lang.org/nightly/core/primitive.reference.html))) -> Self

[Source](../../src/tap/tap.rs.html#429-434)[§](#method.tap_some_mut)

#### fn [tap\_some\_mut](#tymethod.tap_some_mut)(self, func: impl [FnOnce](https://doc.rust-lang.org/nightly/core/ops/function/trait.FnOnce.html "trait core::ops::function::FnOnce")([&mut T](https://doc.rust-lang.org/nightly/core/primitive.reference.html))) -> Self

[Source](../../src/tap/tap.rs.html#437-442)[§](#method.tap_none)

#### fn [tap\_none](#tymethod.tap_none)(self, func: impl [FnOnce](https://doc.rust-lang.org/nightly/core/ops/function/trait.FnOnce.html "trait core::ops::function::FnOnce")()) -> Self

## Implementors[§](#implementors)