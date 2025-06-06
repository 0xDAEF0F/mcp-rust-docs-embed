## [tap](../tap/index.html)1.0.1

## TapFallible

### [Required Associated Types](#required-associated-types)

* [Err](#associatedtype.Err "Err")
* [Ok](#associatedtype.Ok "Ok")

### [Required Methods](#required-methods)

* [tap\_err](#tymethod.tap_err "tap_err")
* [tap\_err\_mut](#tymethod.tap_err_mut "tap_err_mut")
* [tap\_ok](#tymethod.tap_ok "tap_ok")
* [tap\_ok\_mut](#tymethod.tap_ok_mut "tap_ok_mut")

### [Provided Methods](#provided-methods)

* [tap\_err\_dbg](#method.tap_err_dbg "tap_err_dbg")
* [tap\_err\_mut\_dbg](#method.tap_err_mut_dbg "tap_err_mut_dbg")
* [tap\_ok\_dbg](#method.tap_ok_dbg "tap_ok_dbg")
* [tap\_ok\_mut\_dbg](#method.tap_ok_mut_dbg "tap_ok_mut_dbg")

### [Implementations on Foreign Types](#foreign-impls)

* [Result<T, E>](#impl-TapFallible-for-Result%3CT,+E%3E "Result<T, E>")

### [Dyn Compatibility](#dyn-compatibility)

### [Implementors](#implementors)

## [In crate tap](index.html)

[tap](index.html)

# Trait TapFallibleCopy item path

[Source](../src/tap/tap.rs.html#458-550)

```
pub trait TapFallible

where
    Self: Sized,

{
    type Ok: ?Sized;
    type Err: ?Sized;

    // Required methods
    fn tap_ok(self, func: impl FnOnce(&Self::Ok)) -> Self;
    fn tap_ok_mut(self, func: impl FnOnce(&mut Self::Ok)) -> Self;
    fn tap_err(self, func: impl FnOnce(&Self::Err)) -> Self;
    fn tap_err_mut(self, func: impl FnOnce(&mut Self::Err)) -> Self;

    // Provided methods
    fn tap_ok_dbg(self, func: impl FnOnce(&Self::Ok)) -> Self { ... }
    fn tap_ok_mut_dbg(self, func: impl FnOnce(&mut Self::Ok)) -> Self { ... }
    fn tap_err_dbg(self, func: impl FnOnce(&Self::Err)) -> Self { ... }
    fn tap_err_mut_dbg(self, func: impl FnOnce(&mut Self::Err)) -> Self { ... }
}
```

Expand description

Fallible tapping, conditional on the optional success of an expression.

This trait is intended for use on types that express the concept of “fallible
presence”, primarily the [`Result`](https://doc.rust-lang.org/std/result/enum.Result.html) monad. It provides taps that inspect the
container to determine if the effect function should execute or not.

> Note: This trait would ideally be implemented as a blanket over all
> [`std::ops::Try`](https://doc.rust-lang.org/std/ops/trait.Try.html) implementors. When `Try` stabilizes, this crate can be
> updated to do so.

## Required Associated Types[§](#required-associated-types)

[Source](../src/tap/tap.rs.html#463)

#### type [Ok](#associatedtype.Ok): ?[Sized](https://doc.rust-lang.org/nightly/core/marker/trait.Sized.html "trait core::marker::Sized")

The interior type used to indicate a successful construction.

[Source](../src/tap/tap.rs.html#466)

#### type [Err](#associatedtype.Err): ?[Sized](https://doc.rust-lang.org/nightly/core/marker/trait.Sized.html "trait core::marker::Sized")

The interior type used to indicate a failed construction.

## Required Methods[§](#required-methods)

[Source](../src/tap/tap.rs.html#476)

#### fn [tap\_ok](#tymethod.tap_ok)(self, func: impl [FnOnce](https://doc.rust-lang.org/nightly/core/ops/function/trait.FnOnce.html "trait core::ops::function::FnOnce")(&Self::[Ok](trait.TapFallible.html#associatedtype.Ok "type tap::TapFallible::Ok"))) -> Self

Immutably accesses an interior success value.

This function is identical to [`Tap::tap`](trait.Tap.html#method.tap), except that it is required
to check the implementing container for value success before running.
Implementors must not run the effect function if the container is marked
as being a failure.

[Source](../src/tap/tap.rs.html#486)

#### fn [tap\_ok\_mut](#tymethod.tap_ok_mut)(self, func: impl [FnOnce](https://doc.rust-lang.org/nightly/core/ops/function/trait.FnOnce.html "trait core::ops::function::FnOnce")(&mut Self::[Ok](trait.TapFallible.html#associatedtype.Ok "type tap::TapFallible::Ok"))) -> Self

Mutably accesses an interior success value.

This function is identical to [`Tap::tap_mut`](trait.Tap.html#method.tap_mut), except that it is
required to check the implementing container for value success before
running. Implementors must not run the effect function if the container
is marked as being a failure.

[Source](../src/tap/tap.rs.html#496)

#### fn [tap\_err](#tymethod.tap_err)(self, func: impl [FnOnce](https://doc.rust-lang.org/nightly/core/ops/function/trait.FnOnce.html "trait core::ops::function::FnOnce")(&Self::[Err](trait.TapFallible.html#associatedtype.Err "type tap::TapFallible::Err"))) -> Self

Immutably accesses an interior failure value.

This function is identical to [`Tap::tap`](trait.Tap.html#method.tap), except that it is required
to check the implementing container for value failure before running.
Implementors must not run the effect function if the container is marked
as being a success.

[Source](../src/tap/tap.rs.html#506)

#### fn [tap\_err\_mut](#tymethod.tap_err_mut)(self, func: impl [FnOnce](https://doc.rust-lang.org/nightly/core/ops/function/trait.FnOnce.html "trait core::ops::function::FnOnce")(&mut Self::[Err](trait.TapFallible.html#associatedtype.Err "type tap::TapFallible::Err"))) -> Self

Mutably accesses an interior failure value.

This function is identical to [`Tap::tap_mut`](trait.Tap.html#method.tap_mut), except that it is
required to check the implementing container for value failure before
running. Implementors must not run the effect function if the container
is marked as being a success.

## Provided Methods[§](#provided-methods)

[Source](../src/tap/tap.rs.html#510-516)

#### fn [tap\_ok\_dbg](#method.tap_ok_dbg)(self, func: impl [FnOnce](https://doc.rust-lang.org/nightly/core/ops/function/trait.FnOnce.html "trait core::ops::function::FnOnce")(&Self::[Ok](trait.TapFallible.html#associatedtype.Ok "type tap::TapFallible::Ok"))) -> Self

Calls `.tap_ok()` only in debug builds, and is erased in release builds.

[Source](../src/tap/tap.rs.html#521-527)

#### fn [tap\_ok\_mut\_dbg](#method.tap_ok_mut_dbg)(self, func: impl [FnOnce](https://doc.rust-lang.org/nightly/core/ops/function/trait.FnOnce.html "trait core::ops::function::FnOnce")(&mut Self::[Ok](trait.TapFallible.html#associatedtype.Ok "type tap::TapFallible::Ok"))) -> Self

Calls `.tap_ok_mut()` only in debug builds, and is erased in release
builds.

[Source](../src/tap/tap.rs.html#532-538)

#### fn [tap\_err\_dbg](#method.tap_err_dbg)(self, func: impl [FnOnce](https://doc.rust-lang.org/nightly/core/ops/function/trait.FnOnce.html "trait core::ops::function::FnOnce")(&Self::[Err](trait.TapFallible.html#associatedtype.Err "type tap::TapFallible::Err"))) -> Self

Calls `.tap_err()` only in debug builds, and is erased in release
builds.

[Source](../src/tap/tap.rs.html#543-549)

#### fn [tap\_err\_mut\_dbg](#method.tap_err_mut_dbg)(self, func: impl [FnOnce](https://doc.rust-lang.org/nightly/core/ops/function/trait.FnOnce.html "trait core::ops::function::FnOnce")(&mut Self::[Err](trait.TapFallible.html#associatedtype.Err "type tap::TapFallible::Err"))) -> Self

Calls `.tap_err_mut()` only in debug builds, and is erased in release
builds.

## Dyn Compatibility[§](#dyn-compatibility)

This trait is **not** [dyn compatible](https://doc.rust-lang.org/nightly/reference/items/traits.html#dyn-compatibility).

*In older versions of Rust, dyn compatibility was called "object safety", so this trait is not object safe.*

## Implementations on Foreign Types[§](#foreign-impls)

[Source](../src/tap/tap.rs.html#552-587)[§](#impl-TapFallible-for-Result%3CT,+E%3E)

### impl<T, E> [TapFallible](trait.TapFallible.html "trait tap::TapFallible") for [Result](https://doc.rust-lang.org/nightly/core/result/enum.Result.html "enum core::result::Result")<T, E>

[Source](../src/tap/tap.rs.html#553)[§](#associatedtype.Ok-1)

#### type [Ok](#associatedtype.Ok) = T

[Source](../src/tap/tap.rs.html#554)[§](#associatedtype.Err-1)

#### type [Err](#associatedtype.Err) = E

[Source](../src/tap/tap.rs.html#557-562)[§](#method.tap_ok)

#### fn [tap\_ok](#tymethod.tap_ok)(self, func: impl [FnOnce](https://doc.rust-lang.org/nightly/core/ops/function/trait.FnOnce.html "trait core::ops::function::FnOnce")([&T](https://doc.rust-lang.org/nightly/core/primitive.reference.html))) -> Self

[Source](../src/tap/tap.rs.html#565-570)[§](#method.tap_ok_mut)

#### fn [tap\_ok\_mut](#tymethod.tap_ok_mut)(self, func: impl [FnOnce](https://doc.rust-lang.org/nightly/core/ops/function/trait.FnOnce.html "trait core::ops::function::FnOnce")([&mut T](https://doc.rust-lang.org/nightly/core/primitive.reference.html))) -> Self

[Source](../src/tap/tap.rs.html#573-578)[§](#method.tap_err)

#### fn [tap\_err](#tymethod.tap_err)(self, func: impl [FnOnce](https://doc.rust-lang.org/nightly/core/ops/function/trait.FnOnce.html "trait core::ops::function::FnOnce")([&E](https://doc.rust-lang.org/nightly/core/primitive.reference.html))) -> Self

[Source](../src/tap/tap.rs.html#581-586)[§](#method.tap_err_mut)

#### fn [tap\_err\_mut](#tymethod.tap_err_mut)(self, func: impl [FnOnce](https://doc.rust-lang.org/nightly/core/ops/function/trait.FnOnce.html "trait core::ops::function::FnOnce")([&mut E](https://doc.rust-lang.org/nightly/core/primitive.reference.html))) -> Self

## Implementors[§](#implementors)