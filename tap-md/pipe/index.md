## [tap](../../tap/index.html)1.0.1

## Module pipe

### Sections

* [Universal Suffix Calls](#universal-suffix-calls "Universal Suffix Calls")

### [Module Items](#traits)

* [Traits](#traits "Traits")

## [In crate tap](../index.html)

[tap](../index.html)

# Module pipeCopy item path

[Source](../../src/tap/pipe.rs.html#1-234)

Expand description

## [§](#universal-suffix-calls)Universal Suffix Calls

This module provides a single trait, `Pipe`, which provides a number of methods
useful for placing functions in suffix position. The most common method, `pipe`,
forwards a value `T` into any function `T -> R`, returning `R`. The other
methods all apply some form of borrowing to the value before passing the borrow
into the piped function. These are of less value, but provided to maintain a
similar API to the `tap` module’s methods, and for convenience in the event that
you do have a use for them.

This module is as much of a [UFCS](https://en.wikipedia.org/wiki/Uniform_Function_Call_Syntax) method syntax that can be provided as a
library, rather than in the language grammar.

!

## Traits[§](#traits)

[Pipe](trait.Pipe.html "trait tap::pipe::Pipe")
:   Provides universal suffix-position call syntax for any function.