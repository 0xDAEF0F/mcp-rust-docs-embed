## [tap](../../tap/index.html)1.0.1

## Conv

### [Provided Methods](#provided-methods)

* [conv](#method.conv "conv")

### [Dyn Compatibility](#dyn-compatibility)

### [Implementors](#implementors)

## [In tap::prelude](index.html)

[tap](../index.html)::[prelude](index.html)

# Trait ConvCopy item path

[Source](../../src/tap/conv.rs.html#33-56)

```
pub trait Conv

where
    Self: Sized,

{
    // Provided method
    fn conv<T>(self) -> T
       where Self: Into<T>,
             T: Sized { ... }
}
```

Expand description

Wraps `Into::<T>::into` as a method that can be placed in pipelines.

## Provided Methods[§](#provided-methods)

[Source](../../src/tap/conv.rs.html#49-55)

#### fn [conv](#method.conv)<T>(self) -> T where Self: [Into](https://doc.rust-lang.org/nightly/core/convert/trait.Into.html "trait core::convert::Into")<T>, T: [Sized](https://doc.rust-lang.org/nightly/core/marker/trait.Sized.html "trait core::marker::Sized"),

Converts `self` into `T` using `Into<T>`.

##### [§](#examples)Examples

```
use tap::conv::Conv;

let len = "Saluton, mondo!"
  .conv::<String>()
  .len();
```

## Dyn Compatibility[§](#dyn-compatibility)

This trait is **not** [dyn compatible](https://doc.rust-lang.org/nightly/reference/items/traits.html#dyn-compatibility).

*In older versions of Rust, dyn compatibility was called "object safety", so this trait is not object safe.*

## Implementors[§](#implementors)

[Source](../../src/tap/conv.rs.html#58)[§](#impl-Conv-for-T)

### impl<T> [Conv](../trait.Conv.html "trait tap::Conv") for T