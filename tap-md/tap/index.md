## [tap](../../tap/index.html)1.0.1

## Module tap

### Sections

* [Point-Free Inspection](#point-free-inspection "Point-Free Inspection")

### [Module Items](#traits)

* [Traits](#traits "Traits")

## [In crate tap](../index.html)

[tap](../index.html)

# Module tapCopy item path

[Source](../../src/tap/tap.rs.html#1-587)

Expand description

## [§](#point-free-inspection)Point-Free Inspection

The standard library does not provide a way to view or modify an expression
without binding it to a name. This module provides extension methods that take
and return a value, allowing it to be temporarily bound without creating a new
`let`-statement in the enclosing scope.

The two main uses of these methods are to temporarily attach debugging
tracepoints to an expression without modifying its surrounding code, or to
temporarily mutate an otherwise-immutable object.

For convenience, methods are available that will modify the *view* of the tapped
object that is passed to the effect function, by using the value’s
`Borrow`/`BorrowMut`, `AsRef`/`AsMut`, or `Index`/`IndexMut` trait
implementations. For example, the `Vec` collection has no `fn sort` method: this
is actually implemented on slices, to which `Vec` dereferences.

```
use tap::tap::*;

// taps take ordinary closures, which can use deref coercion
make_vec().tap_mut(|v| v.sort());
// `Vec<T>` implements `BorrowMut<[T]>`,
make_vec().tap_borrow_mut(<[_]>::sort);
// and `AsMut<[T]>`,
make_vec().tap_ref_mut(<[_]>::sort);
// and `DerefMut<Target = [T]>,
make_vec().tap_deref_mut(<[_]>::sort);
// but has no inherent method `sort`.
// make_vec().tap_mut(Vec::sort);
```

!

## Traits[§](#traits)

[Tap](trait.Tap.html "trait tap::tap::Tap")
:   Point-free value inspection and modification.

[TapFallible](trait.TapFallible.html "trait tap::tap::TapFallible")
:   Fallible tapping, conditional on the optional success of an expression.

[TapOptional](trait.TapOptional.html "trait tap::tap::TapOptional")
:   Optional tapping, conditional on the optional presence of a value.