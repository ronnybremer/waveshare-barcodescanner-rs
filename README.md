# Experimental crate for interacting with a Waveshare Barcode Scanner Module (UART mode)

[![Build status](https://github.com/ronnybremer/waveshare-barcodescanner-rs/workflows/waveshare-barcodescanner-rust/badge.svg)](https://github.com/ronnybremer/waveshare-barcodescanner-rs/actions/workflows/waveshare-barcodescanner-rust.yml)
[![Documentation](https://docs.rs/waveshare-barcodescanner/badge.svg)](https://docs.rs/waveshare-barcodescanner)
[![Package](https://img.shields.io/crates/v/waveshare-barcodescanner.svg)](https://crates.io/crates/waveshare-barcodescanner)

A high-level interface for interacting with Waveshare Barcode Scanner modules.

The API is usable, but unstable and not very battle-tested; use at your own risk.

[Changelog](https://github.com/ronnybremer/waveshare-barcodescanner-rs/blob/master/CHANGES.md)

### Minimum rustc version (MSRV)

Currently, Rust version 1.85.0 or later is required.

### Hardware support

This crate should work with the following modules:
* [Barcode Scanner Module](htps://www.waveshare.com/barcode-scanner-module.htm)
* [Barcode Scanner Module B](https://www.waveshare.com/barcode-scanner-module-b.htm)
* [Barcode Scanner Module C](https://www.waveshare.com/barcode-scanner-module-c.htm)

Please note that development and testing is done on the `C` revision of the module.

### Platform support

Currently, the main development and testing of the crate is performed on Linux - specifically Raspberry PI 5 on 64-bit Bookworm -, but other major platforms should also work.

### Getting started

Please look at the provided examples on how to use this crate.

### Error handling

All errors are wrapped into `anyhow::Error`.

### Tracing support

This crate is using [Tracing][https://github.com/tokio-rs/tracing].
In your application you need to set up a tracing_subscriber in order to get log output. `DEBUG` and `TRACE` levels are available for troubleshooting.

### Contributing

Contributions are welcome, through issues and PRs.

The license for the original work is [MIT](https://opensource.org/licenses/MIT).
