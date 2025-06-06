## [tap](../tap/index.html)1.0.1

* [All Items](all.html)

### Sections

* [`tap` – Syntactical Plumb-Lines](#tap--syntactical-plumb-lines "`tap` – Syntactical Plumb-Lines")
  + [Tapping](#tapping "Tapping")
  + [Piping](#piping "Piping")
  + [Converting](#converting "Converting")

### [Crate Items](#modules)

* [Modules](#modules "Modules")
* [Traits](#traits "Traits")

# Crate tapCopy item path

[Source](../src/tap/lib.rs.html#1-147)

Expand description

## [§](#tap--syntactical-plumb-lines)`tap` – Syntactical Plumb-Lines

Rust permits functions that take a `self` receiver to be written in “dot-call”
suffix position, rather than the more traditional prefix-position function call
syntax. These functions are restricted to `impl [Trait for] Type` blocks, and
functions anywhere else cannot take advantage of this syntax.

This crate provides universally-implemented extension traits that permit smooth
suffix-position calls for a handful of common operations: transparent inspection
or modification (tapping), transformation (piping), and type conversion.

### [§](#tapping)Tapping

The [`tap`](tap/trait.Tap.html) module provides the [`Tap`](tap/trait.Tap.html), [`TapOptional`](tap/trait.TapOptional.html), and [`TapFallible`](tap/trait.TapFallible.html)
traits. Each of these traits provides methods that take and return a value, and
expose it as a borrow to an effect function. They look like this:

```
use tap::prelude::*;

let end = make_value()
  .tap(|v| log!("Produced value: {:?}", v))
  .process_value();
```

These methods are `self -> Self`, and return the value they received without
any transformation. This enables them to be placed anywhere in a larger
expression witohut changing its shape, or causing any semantic changes to the
code. The effect function receives a borrow of the tapped value, optionally run
through the `Borrow`, `AsRef`, or `Deref` view conversions, for the duration of
its execution.

The effect function cannot return a value, as the tap is incapable of handling
it.

### [§](#piping)Piping

The [`pipe`](pipe/trait.Pipe.html) module provides the [`Pipe`](pipe/trait.Pipe.html) trait. This trait provides methods
that take and transform a value, returning the result of the transformation.
They look like this:

```
use tap::prelude::*;

struct One;
fn start() -> One { One }
struct Two;
fn end(_: One) -> Two { Two }

let val: Two = start().pipe(end);

// without pipes, this would be written as
let _: Two = end(start());
```

These methods are `self -> Other`, and return the value produced by the effect
function. As the methods are always available in suffix position, they can take
as arguments methods that are *not* eligible for dot-call syntax and still place
them as expression suffices. The effect function receives the piped value,
optionally run through the `Borrow`, `AsRef`, or `Deref` view conversions, as
its input, and its output is returned from the pipe.

For `.pipe()`, the input value is *moved* into the pipe and the effect function,
so the effect function *cannot* return a value whose lifetime depends on the
input value. The other pipe methods all borrow the input value, and may return a
value whose lifetime is tied to it.

### [§](#converting)Converting

The [`conv`](conv/trait.Conv.html) module provides the [`Conv`](conv/trait.Conv.html) and [`TryConv`](conv/trait.TryConv.html) traits. These provide
methods that accept a type parameter on the method name, and forward to the
appropriate `Into` or `TryInto` trait implementation when called. The difference
between `Conv` and `Into` is that `Conv` is declared as `Conv::conv::<T>`, while
`Into` is declared as `Into::<T>::into`. The location of the destination type
parameter makes `.into()` unusable as a non-terminal method call of an
expression, while `.conv::<T>()` can be used as a method call anywhere.

ⓘ

```
let upper = "hello, world"
  .into()
  .tap_mut(|s| s.make_ascii_uppercase());
```

The above snippet is illegal, because the Rust type solver cannot determine the
type of the sub-expression `"hello, world".into()`, and it will not attempt to
search all available `impl Into<X> for str` implementations to find an `X` which
has a
`fn tap_mut({self, &self, &mut self, Box<Self>, Rc<Self>, Arc<Self>}, _) -> Y`
declared, either as an inherent method or in a trait implemented by `X`, to
resolve the expression.

Instead, you can write it as

```
use tap::prelude::*;

let upper = "hello, world"
  .conv::<String>()
  .tap_mut(|s| s.make_ascii_uppercase());
```

The trait implementation is

```
pub trait Conv: Sized {
 fn conv<T: Sized>(self) -> T
 where Self: Into<T> {
  self.into()
 }
}
```

Each monomorphization of `.conv::<T>()` expands to the appropriate `Into<T>`
implementation, and does nothing else.

!

## Modules[§](#modules)

[conv](conv/index.html "mod tap::conv")
:   Method-Directed Type Conversion

[pipe](pipe/index.html "mod tap::pipe")
:   Universal Suffix Calls

[prelude](prelude/index.html "mod tap::prelude")
:   Reëxports all traits in one place, for easy import.

[tap](tap/index.html "mod tap::tap")
:   Point-Free Inspection

## Traits[§](#traits)

[Conv](trait.Conv.html "trait tap::Conv")
:   Wraps `Into::<T>::into` as a method that can be placed in pipelines.

[Pipe](trait.Pipe.html "trait tap::Pipe")
:   Provides universal suffix-position call syntax for any function.

[Tap](trait.Tap.html "trait tap::Tap")
:   Point-free value inspection and modification.

[TapFallible](trait.TapFallible.html "trait tap::TapFallible")
:   Fallible tapping, conditional on the optional success of an expression.

[TapOptional](trait.TapOptional.html "trait tap::TapOptional")
:   Optional tapping, conditional on the optional presence of a value.

[TryConv](trait.TryConv.html "trait tap::TryConv")
:   Wraps `TryInto::<T>::try_into` as a method that can be placed in pipelines.