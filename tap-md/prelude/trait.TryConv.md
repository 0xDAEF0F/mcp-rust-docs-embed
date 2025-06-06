## [tap](../../tap/index.html)1.0.1

## TryConv

### [Provided Methods](#provided-methods)

* [try\_conv](#method.try_conv "try_conv")

### [Dyn Compatibility](#dyn-compatibility)

### [Implementors](#implementors)

## [In tap::prelude](index.html)

[tap](../index.html)::[prelude](index.html)

# Trait TryConvCopy item path

[Source](../../src/tap/conv.rs.html#61-85)

```
pub trait TryConv

where
    Self: Sized,

{
    // Provided method
    fn try_conv<T>(self) -> Result<T, Self::Error>
       where Self: TryInto<T>,
             T: Sized { ... }
}
```

Expand description

Wraps `TryInto::<T>::try_into` as a method that can be placed in pipelines.

## Provided Methods[§](#provided-methods)

[Source](../../src/tap/conv.rs.html#78-84)

#### fn [try\_conv](#method.try_conv)<T>(self) -> [Result](https://doc.rust-lang.org/nightly/core/result/enum.Result.html "enum core::result::Result")<T, Self::[Error](https://doc.rust-lang.org/nightly/core/convert/trait.TryInto.html#associatedtype.Error "type core::convert::TryInto::Error")> where Self: [TryInto](https://doc.rust-lang.org/nightly/core/convert/trait.TryInto.html "trait core::convert::TryInto")<T>, T: [Sized](https://doc.rust-lang.org/nightly/core/marker/trait.Sized.html "trait core::marker::Sized"),

Attempts to convert `self` into `T` using `TryInto<T>`.

##### [§](#examples)Examples

```
use tap::conv::TryConv;

let len = "Saluton, mondo!"
  .try_conv::<String>()
  .unwrap()
  .len();
```

## Dyn Compatibility[§](#dyn-compatibility)

This trait is **not** [dyn compatible](https://doc.rust-lang.org/nightly/reference/items/traits.html#dyn-compatibility).

*In older versions of Rust, dyn compatibility was called "object safety", so this trait is not object safe.*

## Implementors[§](#implementors)

[Source](../../src/tap/conv.rs.html#87)[§](#impl-TryConv-for-T)

### impl<T> [TryConv](../trait.TryConv.html "trait tap::TryConv") for T