# randomx4r

This is a fork of https://github.com/moneromint/randomx4r

A Rusty wrapper for RandomX hashing.

## Installation

Depends on the [randomx4r-sys](https://github.com/moneromint/randomx4r-sys) crate for FFI.
Building this crate requires the installation of CMake and a C++ compiler.

## Examples

Some small examples are included in the documentation.
Larger examples are in the examples directory:

* [`multithreaded`](examples/multithreaded.rs) - Hashing with multiple cores.

## Links

* [Latest randomx4r documentation](https://docs.rs/randomx4r/latest/randomx4r)
* [Upstream C++ RandomX implementation](https://github.com/tevador/randomx)

## License

The code in this repository is released under the terms of the MIT license.
See LICENSE file in project root for more info.
