# Tolk-rs

Rust bindings to [Tolk](https://github.com/dkager/tolk), a screen reader abstraction library

## Note

This library is meant to be used in the context of providing a screen reader some text to speak. If you're a sighted developer and want general UI accessibility, this library isn't for you. Please use the more powerful native accessibility APIs provided by the OS. Using this library also means that tools for other disabilities (such as magnifiers, voice dictation, etc.) will not be able to understand your UI.

## Installation

Add this to your Cargo.toml (or use cargo add if you have that):

```toml
tolk = "0.2"
```

## Documentation

Documentation is auto-generated [here](https://docs.rs/tolk/*/x86_64-pc-windows-msvc/tolk/). You don't really need anything else.
