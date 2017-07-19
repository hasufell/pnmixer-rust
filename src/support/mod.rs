//! Support subsystem, with no specific logical coherence.
//!
//! This module provides helper/support functions of various types that
//! don't logically fit elsewhere.

pub mod alsa;
pub mod audio;
pub mod cmd;
pub mod gdk_x11;
#[macro_use]
pub mod glade;
#[macro_use]
pub mod ui;
