# Secure Random for Rust

## Overview

The secure-random crate provides a simple and secure way to generate random numbers in Rust. 
It defines a `SecureRandom` trait for generating random `u64` values and offers a default
implementation using the getrandom crate.

##Features

* Trait Definition: `SecureRandom` trait for generating secure random `u64` numbers.
* Default Implementation: `GetRandom` struct implements `SecureRandom` using the 
 operating system’s random number generator.
* Integration: Seamlessly integrate secure random number generation into your projects.

## Installation

Add secure-random to your Cargo.toml:

```toml
[dependencies]
secure-random = "^0.0.1"
```

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.