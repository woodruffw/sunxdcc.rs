sunxdcc.rs
==========

![license](https://raster.shields.io/badge/license-MIT%20with%20restrictions-green.png)
[![CI](https://github.com/woodruffw/sunxdcc.rs/actions/workflows/ci.yml/badge.svg)](https://github.com/woodruffw/sunxdcc.rs/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/sunxdcc)](https://crates.io/crates/sunxdcc)
[![Documentation](https://docs.rs/sunxdcc/badge.svg)](https://docs.rs/sunxdcc)

A small, unofficial Rust wrapper for the [SunXDCC](https://sunxdcc.com/)
search engine's [API](https://sunxdcc.com/#api).

```rust
use sunxdcc;

for result in sunxdcc::search("hitchhiker's guide to the galaxy") {
  println!("{:?}", result.unwrap());
}
```

See the [documentation](https://docs.rs/sunxdcc) for all available result fields.
