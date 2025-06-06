## [tap](../../tap/index.html)1.0.1

## Module prelude

### [Module Items](#traits)

* [Traits](#traits "Traits")

## [In crate tap](../index.html)

[tap](../index.html)

# Module preludeCopy item path

[Source](../../src/tap/lib.rs.html#140)

Expand description

Reëxports all traits in one place, for easy import.

## Traits[§](#traits)

[Conv](trait.Conv.html "trait tap::prelude::Conv")
:   Wraps `Into::<T>::into` as a method that can be placed in pipelines.

[Pipe](trait.Pipe.html "trait tap::prelude::Pipe")
:   Provides universal suffix-position call syntax for any function.

[Tap](trait.Tap.html "trait tap::prelude::Tap")
:   Point-free value inspection and modification.

[TapFallible](trait.TapFallible.html "trait tap::prelude::TapFallible")
:   Fallible tapping, conditional on the optional success of an expression.

[TapOptional](trait.TapOptional.html "trait tap::prelude::TapOptional")
:   Optional tapping, conditional on the optional presence of a value.

[TryConv](trait.TryConv.html "trait tap::prelude::TryConv")
:   Wraps `TryInto::<T>::try_into` as a method that can be placed in pipelines.