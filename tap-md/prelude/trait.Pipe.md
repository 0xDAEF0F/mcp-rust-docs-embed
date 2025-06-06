## [tap](../../tap/index.html)1.0.1

## Pipe

### [Provided Methods](#provided-methods)

* [pipe](#method.pipe "pipe")
* [pipe\_as\_mut](#method.pipe_as_mut "pipe_as_mut")
* [pipe\_as\_ref](#method.pipe_as_ref "pipe_as_ref")
* [pipe\_borrow](#method.pipe_borrow "pipe_borrow")
* [pipe\_borrow\_mut](#method.pipe_borrow_mut "pipe_borrow_mut")
* [pipe\_deref](#method.pipe_deref "pipe_deref")
* [pipe\_deref\_mut](#method.pipe_deref_mut "pipe_deref_mut")
* [pipe\_ref](#method.pipe_ref "pipe_ref")
* [pipe\_ref\_mut](#method.pipe_ref_mut "pipe_ref_mut")

### [Dyn Compatibility](#dyn-compatibility)

### [Implementors](#implementors)

## [In tap::prelude](index.html)

[tap](../index.html)::[prelude](index.html)

# Trait PipeCopy item path

[Source](../../src/tap/pipe.rs.html#55-232)

```
pub trait Pipe {
    // Provided methods
    fn pipe<R>(self, func: impl FnOnce(Self) -> R) -> R
       where Self: Sized,
             R: Sized { ... }
    fn pipe_ref<'a, R>(&'a self, func: impl FnOnce(&'a Self) -> R) -> R
       where R: 'a + Sized { ... }
    fn pipe_ref_mut<'a, R>(
        &'a mut self,
        func: impl FnOnce(&'a mut Self) -> R,
    ) -> R
       where R: 'a + Sized { ... }
    fn pipe_borrow<'a, B, R>(&'a self, func: impl FnOnce(&'a B) -> R) -> R
       where Self: Borrow<B>,
             B: 'a + ?Sized,
             R: 'a + Sized { ... }
    fn pipe_borrow_mut<'a, B, R>(
        &'a mut self,
        func: impl FnOnce(&'a mut B) -> R,
    ) -> R
       where Self: BorrowMut<B>,
             B: 'a + ?Sized,
             R: 'a + Sized { ... }
    fn pipe_as_ref<'a, U, R>(&'a self, func: impl FnOnce(&'a U) -> R) -> R
       where Self: AsRef<U>,
             U: 'a + ?Sized,
             R: 'a + Sized { ... }
    fn pipe_as_mut<'a, U, R>(
        &'a mut self,
        func: impl FnOnce(&'a mut U) -> R,
    ) -> R
       where Self: AsMut<U>,
             U: 'a + ?Sized,
             R: 'a + Sized { ... }
    fn pipe_deref<'a, T, R>(&'a self, func: impl FnOnce(&'a T) -> R) -> R
       where Self: Deref<Target = T>,
             T: 'a + ?Sized,
             R: 'a + Sized { ... }
    fn pipe_deref_mut<'a, T, R>(
        &'a mut self,
        func: impl FnOnce(&'a mut T) -> R,
    ) -> R
       where Self: DerefMut + Deref<Target = T>,
             T: 'a + ?Sized,
             R: 'a + Sized { ... }
}
```

Expand description

Provides universal suffix-position call syntax for any function.

This trait provides methods that allow any closure or free function to be placed
as a suffix-position call, by writing them as

```
fn not_a_method(x: i32) -> u8 { x as u8 }
receiver.pipe(not_a_method);
```

Piping into functions that take more than one argument still requires writing a
closure with ordinary function-call syntax. This is after all only a library,
not a syntax transformation:

```
use tap::pipe::Pipe;
fn add(x: i32, y: i32) -> i32 { x + y }

let out = 5.pipe(|x| add(x, 10));
assert_eq!(out, 15);
```

Like tapping, piping is useful for cases where you want to write a sequence of
processing steps without introducing many intermediate bindings, and your steps
contain functions which are not eligible for dot-call syntax.

The main difference between piping and tapping is that tapping always returns
the value that was passed into the tap, while piping forwards the value into the
effect function, and returns the output of evaluating the effect function with
the value. Piping is a transformation, not merely an inspection or modification.

## Provided Methods[§](#provided-methods)

[Source](../../src/tap/pipe.rs.html#73-79)

#### fn [pipe](#method.pipe)<R>(self, func: impl [FnOnce](https://doc.rust-lang.org/nightly/core/ops/function/trait.FnOnce.html "trait core::ops::function::FnOnce")(Self) -> R) -> R where Self: [Sized](https://doc.rust-lang.org/nightly/core/marker/trait.Sized.html "trait core::marker::Sized"), R: [Sized](https://doc.rust-lang.org/nightly/core/marker/trait.Sized.html "trait core::marker::Sized"),

Pipes by value. This is generally the method you want to use.

##### [§](#examples)Examples

```
use tap::pipe::Pipe;

fn triple(x: i32) -> i64 {
  x as i64 * 3
}

assert_eq!(
  10.pipe(triple),
  30,
);
```

[Source](../../src/tap/pipe.rs.html#97-102)

#### fn [pipe\_ref](#method.pipe_ref)<'a, R>(&'a self, func: impl [FnOnce](https://doc.rust-lang.org/nightly/core/ops/function/trait.FnOnce.html "trait core::ops::function::FnOnce")(&'a Self) -> R) -> R where R: 'a + [Sized](https://doc.rust-lang.org/nightly/core/marker/trait.Sized.html "trait core::marker::Sized"),

Borrows `self` and passes that borrow into the pipe function.

##### [§](#examples-1)Examples

```
use tap::pipe::Pipe;

fn fold(v: &Vec<i32>) -> i32 {
  v.iter().copied().sum()
}
let vec = vec![1, 2, 3, 4, 5];
let sum = vec.pipe_ref(fold);
assert_eq!(sum, 15);
assert_eq!(vec.len(), 5);
```

[Source](../../src/tap/pipe.rs.html#122-130)

#### fn [pipe\_ref\_mut](#method.pipe_ref_mut)<'a, R>(&'a mut self, func: impl [FnOnce](https://doc.rust-lang.org/nightly/core/ops/function/trait.FnOnce.html "trait core::ops::function::FnOnce")(&'a mut Self) -> R) -> R where R: 'a + [Sized](https://doc.rust-lang.org/nightly/core/marker/trait.Sized.html "trait core::marker::Sized"),

Mutably borrows `self` and passes that borrow into the pipe function.

##### [§](#examples-2)Examples

```
use tap::pipe::Pipe;

let mut vec = vec![false, true];
let last = vec
  .pipe_ref_mut(Vec::pop)
  .pipe(Option::unwrap);
assert!(last);
```

Both of these functions are eligible for method-call syntax, and should
not be piped. Writing out non-trivial examples for these is a lot of
boilerplate.

[Source](../../src/tap/pipe.rs.html#145-152)

#### fn [pipe\_borrow](#method.pipe_borrow)<'a, B, R>(&'a self, func: impl [FnOnce](https://doc.rust-lang.org/nightly/core/ops/function/trait.FnOnce.html "trait core::ops::function::FnOnce")([&'a B](https://doc.rust-lang.org/nightly/core/primitive.reference.html)) -> R) -> R where Self: [Borrow](https://doc.rust-lang.org/nightly/core/borrow/trait.Borrow.html "trait core::borrow::Borrow")<B>, B: 'a + ?[Sized](https://doc.rust-lang.org/nightly/core/marker/trait.Sized.html "trait core::marker::Sized"), R: 'a + [Sized](https://doc.rust-lang.org/nightly/core/marker/trait.Sized.html "trait core::marker::Sized"),

Borrows `self`, then passes `self.borrow()` into the pipe function.

##### [§](#examples-3)Examples

```
use std::borrow::Cow;
use tap::pipe::Pipe;

let len = Cow::<'static, str>::from("hello, world")
  .pipe_borrow(str::len);
assert_eq!(len, 12);
```

[Source](../../src/tap/pipe.rs.html#169-179)

#### fn [pipe\_borrow\_mut](#method.pipe_borrow_mut)<'a, B, R>( &'a mut self, func: impl [FnOnce](https://doc.rust-lang.org/nightly/core/ops/function/trait.FnOnce.html "trait core::ops::function::FnOnce")([&'a mut B](https://doc.rust-lang.org/nightly/core/primitive.reference.html)) -> R, ) -> R where Self: [BorrowMut](https://doc.rust-lang.org/nightly/core/borrow/trait.BorrowMut.html "trait core::borrow::BorrowMut")<B>, B: 'a + ?[Sized](https://doc.rust-lang.org/nightly/core/marker/trait.Sized.html "trait core::marker::Sized"), R: 'a + [Sized](https://doc.rust-lang.org/nightly/core/marker/trait.Sized.html "trait core::marker::Sized"),

Mutably borrows `self`, then passes `self.borrow_mut()` into the pipe
function.

```
use tap::pipe::Pipe;

let mut txt = "hello, world".to_string();
let ptr = txt
  .pipe_borrow_mut(str::as_mut_ptr);
```

This is a very contrived example, but the `BorrowMut` trait has almost
no implementors in the standard library, and of the implementations
available, there are almost no methods that fit this API.

[Source](../../src/tap/pipe.rs.html#183-190)

#### fn [pipe\_as\_ref](#method.pipe_as_ref)<'a, U, R>(&'a self, func: impl [FnOnce](https://doc.rust-lang.org/nightly/core/ops/function/trait.FnOnce.html "trait core::ops::function::FnOnce")([&'a U](https://doc.rust-lang.org/nightly/core/primitive.reference.html)) -> R) -> R where Self: [AsRef](https://doc.rust-lang.org/nightly/core/convert/trait.AsRef.html "trait core::convert::AsRef")<U>, U: 'a + ?[Sized](https://doc.rust-lang.org/nightly/core/marker/trait.Sized.html "trait core::marker::Sized"), R: 'a + [Sized](https://doc.rust-lang.org/nightly/core/marker/trait.Sized.html "trait core::marker::Sized"),

Borrows `self`, then passes `self.as_ref()` into the pipe function.

[Source](../../src/tap/pipe.rs.html#195-205)

#### fn [pipe\_as\_mut](#method.pipe_as_mut)<'a, U, R>(&'a mut self, func: impl [FnOnce](https://doc.rust-lang.org/nightly/core/ops/function/trait.FnOnce.html "trait core::ops::function::FnOnce")([&'a mut U](https://doc.rust-lang.org/nightly/core/primitive.reference.html)) -> R) -> R where Self: [AsMut](https://doc.rust-lang.org/nightly/core/convert/trait.AsMut.html "trait core::convert::AsMut")<U>, U: 'a + ?[Sized](https://doc.rust-lang.org/nightly/core/marker/trait.Sized.html "trait core::marker::Sized"), R: 'a + [Sized](https://doc.rust-lang.org/nightly/core/marker/trait.Sized.html "trait core::marker::Sized"),

Mutably borrows `self`, then passes `self.as_mut()` into the pipe
function.

[Source](../../src/tap/pipe.rs.html#209-216)

#### fn [pipe\_deref](#method.pipe_deref)<'a, T, R>(&'a self, func: impl [FnOnce](https://doc.rust-lang.org/nightly/core/ops/function/trait.FnOnce.html "trait core::ops::function::FnOnce")([&'a T](https://doc.rust-lang.org/nightly/core/primitive.reference.html)) -> R) -> R where Self: [Deref](https://doc.rust-lang.org/nightly/core/ops/deref/trait.Deref.html "trait core::ops::deref::Deref")<Target = T>, T: 'a + ?[Sized](https://doc.rust-lang.org/nightly/core/marker/trait.Sized.html "trait core::marker::Sized"), R: 'a + [Sized](https://doc.rust-lang.org/nightly/core/marker/trait.Sized.html "trait core::marker::Sized"),

Borrows `self`, then passes `self.deref()` into the pipe function.

[Source](../../src/tap/pipe.rs.html#221-231)

#### fn [pipe\_deref\_mut](#method.pipe_deref_mut)<'a, T, R>( &'a mut self, func: impl [FnOnce](https://doc.rust-lang.org/nightly/core/ops/function/trait.FnOnce.html "trait core::ops::function::FnOnce")([&'a mut T](https://doc.rust-lang.org/nightly/core/primitive.reference.html)) -> R, ) -> R where Self: [DerefMut](https://doc.rust-lang.org/nightly/core/ops/deref/trait.DerefMut.html "trait core::ops::deref::DerefMut") + [Deref](https://doc.rust-lang.org/nightly/core/ops/deref/trait.Deref.html "trait core::ops::deref::Deref")<Target = T>, T: 'a + ?[Sized](https://doc.rust-lang.org/nightly/core/marker/trait.Sized.html "trait core::marker::Sized"), R: 'a + [Sized](https://doc.rust-lang.org/nightly/core/marker/trait.Sized.html "trait core::marker::Sized"),

Mutably borrows `self`, then passes `self.deref_mut()` into the pipe
function.

## Dyn Compatibility[§](#dyn-compatibility)

This trait is **not** [dyn compatible](https://doc.rust-lang.org/nightly/reference/items/traits.html#dyn-compatibility).

*In older versions of Rust, dyn compatibility was called "object safety", so this trait is not object safe.*

## Implementors[§](#implementors)

[Source](../../src/tap/pipe.rs.html#234)[§](#impl-Pipe-for-T)

### impl<T> [Pipe](../trait.Pipe.html "trait tap::Pipe") for T where T: ?[Sized](https://doc.rust-lang.org/nightly/core/marker/trait.Sized.html "trait core::marker::Sized"),