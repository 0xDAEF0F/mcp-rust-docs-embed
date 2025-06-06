## [tap](../../tap/index.html)1.0.1

## Tap

### [Provided Methods](#provided-methods)

* [tap](#method.tap "tap")
* [tap\_borrow](#method.tap_borrow "tap_borrow")
* [tap\_borrow\_dbg](#method.tap_borrow_dbg "tap_borrow_dbg")
* [tap\_borrow\_mut](#method.tap_borrow_mut "tap_borrow_mut")
* [tap\_borrow\_mut\_dbg](#method.tap_borrow_mut_dbg "tap_borrow_mut_dbg")
* [tap\_dbg](#method.tap_dbg "tap_dbg")
* [tap\_deref](#method.tap_deref "tap_deref")
* [tap\_deref\_dbg](#method.tap_deref_dbg "tap_deref_dbg")
* [tap\_deref\_mut](#method.tap_deref_mut "tap_deref_mut")
* [tap\_deref\_mut\_dbg](#method.tap_deref_mut_dbg "tap_deref_mut_dbg")
* [tap\_mut](#method.tap_mut "tap_mut")
* [tap\_mut\_dbg](#method.tap_mut_dbg "tap_mut_dbg")
* [tap\_ref](#method.tap_ref "tap_ref")
* [tap\_ref\_dbg](#method.tap_ref_dbg "tap_ref_dbg")
* [tap\_ref\_mut](#method.tap_ref_mut "tap_ref_mut")
* [tap\_ref\_mut\_dbg](#method.tap_ref_mut_dbg "tap_ref_mut_dbg")

### [Dyn Compatibility](#dyn-compatibility)

### [Implementors](#implementors)

## [In tap::prelude](index.html)

[tap](../index.html)::[prelude](index.html)

# Trait TapCopy item path

[Source](../../src/tap/tap.rs.html#49-327)

```
pub trait Tap

where
    Self: Sized,

{
Show 16 methods    // Provided methods
    fn tap(self, func: impl FnOnce(&Self)) -> Self { ... }
    fn tap_mut(self, func: impl FnOnce(&mut Self)) -> Self { ... }
    fn tap_borrow<B>(self, func: impl FnOnce(&B)) -> Self
       where Self: Borrow<B>,
             B: ?Sized { ... }
    fn tap_borrow_mut<B>(self, func: impl FnOnce(&mut B)) -> Self
       where Self: BorrowMut<B>,
             B: ?Sized { ... }
    fn tap_ref<R>(self, func: impl FnOnce(&R)) -> Self
       where Self: AsRef<R>,
             R: ?Sized { ... }
    fn tap_ref_mut<R>(self, func: impl FnOnce(&mut R)) -> Self
       where Self: AsMut<R>,
             R: ?Sized { ... }
    fn tap_deref<T>(self, func: impl FnOnce(&T)) -> Self
       where Self: Deref<Target = T>,
             T: ?Sized { ... }
    fn tap_deref_mut<T>(self, func: impl FnOnce(&mut T)) -> Self
       where Self: DerefMut + Deref<Target = T>,
             T: ?Sized { ... }
    fn tap_dbg(self, func: impl FnOnce(&Self)) -> Self { ... }
    fn tap_mut_dbg(self, func: impl FnOnce(&mut Self)) -> Self { ... }
    fn tap_borrow_dbg<B>(self, func: impl FnOnce(&B)) -> Self
       where Self: Borrow<B>,
             B: ?Sized { ... }
    fn tap_borrow_mut_dbg<B>(self, func: impl FnOnce(&mut B)) -> Self
       where Self: BorrowMut<B>,
             B: ?Sized { ... }
    fn tap_ref_dbg<R>(self, func: impl FnOnce(&R)) -> Self
       where Self: AsRef<R>,
             R: ?Sized { ... }
    fn tap_ref_mut_dbg<R>(self, func: impl FnOnce(&mut R)) -> Self
       where Self: AsMut<R>,
             R: ?Sized { ... }
    fn tap_deref_dbg<T>(self, func: impl FnOnce(&T)) -> Self
       where Self: Deref<Target = T>,
             T: ?Sized { ... }
    fn tap_deref_mut_dbg<T>(self, func: impl FnOnce(&mut T)) -> Self
       where Self: DerefMut + Deref<Target = T>,
             T: ?Sized { ... }
}
```

Expand description

Point-free value inspection and modification.

This trait provides methods that permit viewing the value of an expression
without requiring a new `let` binding or any other alterations to the original
code other than insertion of the `.tap()` call.

The methods in this trait do not perform any view conversions on the value they
receive; it is borrowed and passed directly to the effect argument.

## Provided Methods[§](#provided-methods)

[Source](../../src/tap/tap.rs.html#78-81)

#### fn [tap](#method.tap)(self, func: impl [FnOnce](https://doc.rust-lang.org/nightly/core/ops/function/trait.FnOnce.html "trait core::ops::function::FnOnce")(&Self)) -> Self

Immutable access to a value.

This function permits a value to be viewed by some inspecting function
without affecting the overall shape of the expression that contains this
method call. It is useful for attaching assertions or logging points
into a multi-part expression.

##### [§](#examples)Examples

Here we use `.tap()` to attach logging tracepoints to each stage of a
value-processing pipeline.

```
use tap::tap::Tap;

let end = make_value()
  // this line has no effect on the rest of the code
  .tap(|v| log!("The produced value was: {}", v))
  .process_value();
```

[Source](../../src/tap/tap.rs.html#116-119)

#### fn [tap\_mut](#method.tap_mut)(self, func: impl [FnOnce](https://doc.rust-lang.org/nightly/core/ops/function/trait.FnOnce.html "trait core::ops::function::FnOnce")(&mut Self)) -> Self

Mutable access to a value.

This function permits a value to be modified by some function without
affecting the overall shape of the expression that contains this method
call. It is useful for attaching modifier functions that have an
`&mut Self -> ()` signature to an expression, without requiring an
explicit `let mut` binding.

##### [§](#examples-1)Examples

Here we use `.tap_mut()` to sort an array without requring multiple
bindings.

```
use tap::tap::Tap;

let sorted = [1i32, 5, 2, 4, 3]
  .tap_mut(|arr| arr.sort());
assert_eq!(sorted, [1, 2, 3, 4, 5]);
```

Without tapping, this would be written as

```
let mut received = [1, 5, 2, 4, 3];
received.sort();
let sorted = received;
```

The mutable tap is a convenient alternative when the expression to
produce the collection is more complex, for example, an iterator
pipeline collected into a vector.

[Source](../../src/tap/tap.rs.html#129-136)

#### fn [tap\_borrow](#method.tap_borrow)<B>(self, func: impl [FnOnce](https://doc.rust-lang.org/nightly/core/ops/function/trait.FnOnce.html "trait core::ops::function::FnOnce")([&B](https://doc.rust-lang.org/nightly/core/primitive.reference.html))) -> Self where Self: [Borrow](https://doc.rust-lang.org/nightly/core/borrow/trait.Borrow.html "trait core::borrow::Borrow")<B>, B: ?[Sized](https://doc.rust-lang.org/nightly/core/marker/trait.Sized.html "trait core::marker::Sized"),

Immutable access to the `Borrow<B>` of a value.

This function is identcal to [`Tap::tap`](trait.Tap.html#method.tap), except that the effect
function recevies an `&B` produced by `Borrow::<B>::borrow`, rather than
an `&Self`.

[Source](../../src/tap/tap.rs.html#146-153)

#### fn [tap\_borrow\_mut](#method.tap_borrow_mut)<B>(self, func: impl [FnOnce](https://doc.rust-lang.org/nightly/core/ops/function/trait.FnOnce.html "trait core::ops::function::FnOnce")([&mut B](https://doc.rust-lang.org/nightly/core/primitive.reference.html))) -> Self where Self: [BorrowMut](https://doc.rust-lang.org/nightly/core/borrow/trait.BorrowMut.html "trait core::borrow::BorrowMut")<B>, B: ?[Sized](https://doc.rust-lang.org/nightly/core/marker/trait.Sized.html "trait core::marker::Sized"),

Mutable access to the `BorrowMut<B>` of a value.

This function is identical to [`Tap::tap_mut`](trait.Tap.html#method.tap_mut), except that the effect
function receives an `&mut B` produced by `BorrowMut::<B>::borrow_mut`,
rather than an `&mut Self`.

[Source](../../src/tap/tap.rs.html#163-170)

#### fn [tap\_ref](#method.tap_ref)<R>(self, func: impl [FnOnce](https://doc.rust-lang.org/nightly/core/ops/function/trait.FnOnce.html "trait core::ops::function::FnOnce")([&R](https://doc.rust-lang.org/nightly/core/primitive.reference.html))) -> Self where Self: [AsRef](https://doc.rust-lang.org/nightly/core/convert/trait.AsRef.html "trait core::convert::AsRef")<R>, R: ?[Sized](https://doc.rust-lang.org/nightly/core/marker/trait.Sized.html "trait core::marker::Sized"),

Immutable access to the `AsRef<R>` view of a value.

This function is identical to [`Tap::tap`](trait.Tap.html#method.tap), except that the effect
function receives an `&R` produced by `AsRef::<R>::as_ref`, rather than
an `&Self`.

[Source](../../src/tap/tap.rs.html#180-187)

#### fn [tap\_ref\_mut](#method.tap_ref_mut)<R>(self, func: impl [FnOnce](https://doc.rust-lang.org/nightly/core/ops/function/trait.FnOnce.html "trait core::ops::function::FnOnce")([&mut R](https://doc.rust-lang.org/nightly/core/primitive.reference.html))) -> Self where Self: [AsMut](https://doc.rust-lang.org/nightly/core/convert/trait.AsMut.html "trait core::convert::AsMut")<R>, R: ?[Sized](https://doc.rust-lang.org/nightly/core/marker/trait.Sized.html "trait core::marker::Sized"),

Mutable access to the `AsMut<R>` view of a value.

This function is identical to [`Tap::tap_mut`](trait.Tap.html#method.tap_mut), except that the effect
function receives an `&mut R` produced by `AsMut::<R>::as_mut`, rather
than an `&mut Self`.

[Source](../../src/tap/tap.rs.html#197-204)

#### fn [tap\_deref](#method.tap_deref)<T>(self, func: impl [FnOnce](https://doc.rust-lang.org/nightly/core/ops/function/trait.FnOnce.html "trait core::ops::function::FnOnce")([&T](https://doc.rust-lang.org/nightly/core/primitive.reference.html))) -> Self where Self: [Deref](https://doc.rust-lang.org/nightly/core/ops/deref/trait.Deref.html "trait core::ops::deref::Deref")<Target = T>, T: ?[Sized](https://doc.rust-lang.org/nightly/core/marker/trait.Sized.html "trait core::marker::Sized"),

Immutable access to the `Deref::Target` of a value.

This function is identical to [`Tap::tap`](trait.Tap.html#method.tap), except that the effect
function receives an `&Self::Target` produced by `Deref::deref`, rather
than an `&Self`.

[Source](../../src/tap/tap.rs.html#214-221)

#### fn [tap\_deref\_mut](#method.tap_deref_mut)<T>(self, func: impl [FnOnce](https://doc.rust-lang.org/nightly/core/ops/function/trait.FnOnce.html "trait core::ops::function::FnOnce")([&mut T](https://doc.rust-lang.org/nightly/core/primitive.reference.html))) -> Self where Self: [DerefMut](https://doc.rust-lang.org/nightly/core/ops/deref/trait.DerefMut.html "trait core::ops::deref::DerefMut") + [Deref](https://doc.rust-lang.org/nightly/core/ops/deref/trait.Deref.html "trait core::ops::deref::Deref")<Target = T>, T: ?[Sized](https://doc.rust-lang.org/nightly/core/marker/trait.Sized.html "trait core::marker::Sized"),

Mutable access to the `Deref::Target` of a value.

This function is identical to [`Tap::tap_mut`](trait.Tap.html#method.tap_mut), except that the effect
function receives an `&mut Self::Target` produced by
`DerefMut::deref_mut`, rather than an `&mut Self`.

[Source](../../src/tap/tap.rs.html#227-232)

#### fn [tap\_dbg](#method.tap_dbg)(self, func: impl [FnOnce](https://doc.rust-lang.org/nightly/core/ops/function/trait.FnOnce.html "trait core::ops::function::FnOnce")(&Self)) -> Self

Calls `.tap()` only in debug builds, and is erased in release builds.

[Source](../../src/tap/tap.rs.html#237-242)

#### fn [tap\_mut\_dbg](#method.tap_mut_dbg)(self, func: impl [FnOnce](https://doc.rust-lang.org/nightly/core/ops/function/trait.FnOnce.html "trait core::ops::function::FnOnce")(&mut Self)) -> Self

Calls `.tap_mut()` only in debug builds, and is erased in release
builds.

[Source](../../src/tap/tap.rs.html#247-256)

#### fn [tap\_borrow\_dbg](#method.tap_borrow_dbg)<B>(self, func: impl [FnOnce](https://doc.rust-lang.org/nightly/core/ops/function/trait.FnOnce.html "trait core::ops::function::FnOnce")([&B](https://doc.rust-lang.org/nightly/core/primitive.reference.html))) -> Self where Self: [Borrow](https://doc.rust-lang.org/nightly/core/borrow/trait.Borrow.html "trait core::borrow::Borrow")<B>, B: ?[Sized](https://doc.rust-lang.org/nightly/core/marker/trait.Sized.html "trait core::marker::Sized"),

Calls `.tap_borrow()` only in debug builds, and is erased in release
builds.

[Source](../../src/tap/tap.rs.html#261-270)

#### fn [tap\_borrow\_mut\_dbg](#method.tap_borrow_mut_dbg)<B>(self, func: impl [FnOnce](https://doc.rust-lang.org/nightly/core/ops/function/trait.FnOnce.html "trait core::ops::function::FnOnce")([&mut B](https://doc.rust-lang.org/nightly/core/primitive.reference.html))) -> Self where Self: [BorrowMut](https://doc.rust-lang.org/nightly/core/borrow/trait.BorrowMut.html "trait core::borrow::BorrowMut")<B>, B: ?[Sized](https://doc.rust-lang.org/nightly/core/marker/trait.Sized.html "trait core::marker::Sized"),

Calls `.tap_borrow_mut()` only in debug builds, and is erased in release
builds.

[Source](../../src/tap/tap.rs.html#275-284)

#### fn [tap\_ref\_dbg](#method.tap_ref_dbg)<R>(self, func: impl [FnOnce](https://doc.rust-lang.org/nightly/core/ops/function/trait.FnOnce.html "trait core::ops::function::FnOnce")([&R](https://doc.rust-lang.org/nightly/core/primitive.reference.html))) -> Self where Self: [AsRef](https://doc.rust-lang.org/nightly/core/convert/trait.AsRef.html "trait core::convert::AsRef")<R>, R: ?[Sized](https://doc.rust-lang.org/nightly/core/marker/trait.Sized.html "trait core::marker::Sized"),

Calls `.tap_ref()` only in debug builds, and is erased in release
builds.

[Source](../../src/tap/tap.rs.html#289-298)

#### fn [tap\_ref\_mut\_dbg](#method.tap_ref_mut_dbg)<R>(self, func: impl [FnOnce](https://doc.rust-lang.org/nightly/core/ops/function/trait.FnOnce.html "trait core::ops::function::FnOnce")([&mut R](https://doc.rust-lang.org/nightly/core/primitive.reference.html))) -> Self where Self: [AsMut](https://doc.rust-lang.org/nightly/core/convert/trait.AsMut.html "trait core::convert::AsMut")<R>, R: ?[Sized](https://doc.rust-lang.org/nightly/core/marker/trait.Sized.html "trait core::marker::Sized"),

Calls `.tap_ref_mut()` only in debug builds, and is erased in release
builds.

[Source](../../src/tap/tap.rs.html#303-312)

#### fn [tap\_deref\_dbg](#method.tap_deref_dbg)<T>(self, func: impl [FnOnce](https://doc.rust-lang.org/nightly/core/ops/function/trait.FnOnce.html "trait core::ops::function::FnOnce")([&T](https://doc.rust-lang.org/nightly/core/primitive.reference.html))) -> Self where Self: [Deref](https://doc.rust-lang.org/nightly/core/ops/deref/trait.Deref.html "trait core::ops::deref::Deref")<Target = T>, T: ?[Sized](https://doc.rust-lang.org/nightly/core/marker/trait.Sized.html "trait core::marker::Sized"),

Calls `.tap_deref()` only in debug builds, and is erased in release
builds.

[Source](../../src/tap/tap.rs.html#317-326)

#### fn [tap\_deref\_mut\_dbg](#method.tap_deref_mut_dbg)<T>(self, func: impl [FnOnce](https://doc.rust-lang.org/nightly/core/ops/function/trait.FnOnce.html "trait core::ops::function::FnOnce")([&mut T](https://doc.rust-lang.org/nightly/core/primitive.reference.html))) -> Self where Self: [DerefMut](https://doc.rust-lang.org/nightly/core/ops/deref/trait.DerefMut.html "trait core::ops::deref::DerefMut") + [Deref](https://doc.rust-lang.org/nightly/core/ops/deref/trait.Deref.html "trait core::ops::deref::Deref")<Target = T>, T: ?[Sized](https://doc.rust-lang.org/nightly/core/marker/trait.Sized.html "trait core::marker::Sized"),

Calls `.tap_deref_mut()` only in debug builds, and is erased in release
builds.

## Dyn Compatibility[§](#dyn-compatibility)

This trait is **not** [dyn compatible](https://doc.rust-lang.org/nightly/reference/items/traits.html#dyn-compatibility).

*In older versions of Rust, dyn compatibility was called "object safety", so this trait is not object safe.*

## Implementors[§](#implementors)

[Source](../../src/tap/tap.rs.html#329)[§](#impl-Tap-for-T)

### impl<T> [Tap](../trait.Tap.html "trait tap::Tap") for T where T: [Sized](https://doc.rust-lang.org/nightly/core/marker/trait.Sized.html "trait core::marker::Sized"),