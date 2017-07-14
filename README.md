[![Build Status](https://travis-ci.org/hasufell/pnmixer-rust.svg)](https://travis-ci.org/hasufell/pnmixer-rust)
[![Join the chat at https://gitter.im/hasufell/pnmixer-rust](https://badges.gitter.im/hasufell/pnmixer-rust.svg)](https://gitter.im/hasufell/pnmixer-rust?utm_source=badge&utm_medium=badge&utm_campaign=pr-badge&utm_content=badge)
[![Documentation (master)](https://img.shields.io/badge/documentation-master-yellow.svg)](https://hasufell.github.io/pnmixer-rust/pnmixerlib/)
[![License](https://img.shields.io/github/license/hasufell/pnmixer-rust.svg)](https://github.com/hasufell/pnmixer-rust)

PNMixer-rs
==========

About
-----

Rewrite of [nicklan/pnmixer](https://github.com/nicklan/pnmixer) in
[Rust](https://www.rust-lang.org).

This is meant as a drop-in replacement, but may diverge in feature set
in the future.

Installation
------------

The Rust ecosystem uses [Cargo](https://crates.io/), as such, you need
both the rust compiler and the cargo crate
(usually part of the compiler toolchain), then issue from within
the cloned repository:

```sh
cargo install
```

Features
--------

Additonal features compared to [nicklan/pnmixer](https://github.com/nicklan/pnmixer):

* decide whether to unmute or not on explicit volume change
* updates tray icon on icon theme change

Removed features:

* normalize volume
* slider orientation of volume popup
* settings for displaying text volume on volume popup

Behavior
--------

Pretty much the same. Differences are:

* volume slider is shown even when volume is muted

TODO
----

- [ ] [hotkey support](https://github.com/hasufell/pnmixer-rust/issues/5)
- [ ] [translation](https://github.com/hasufell/pnmixer-rust/issues/4)
- [ ] [documentation](https://github.com/hasufell/pnmixer-rust/issues/3)
