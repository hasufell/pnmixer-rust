//! PNMixer-rs is a mixer for the system tray.
//!
//! # Design Overview
//!
//! The lowest level part of the code is the sound backend. Only Alsa is supported
//! at the moment, but more backends may be added in the future.
//!
//! The backend is hidden behind a frontend, defined in `audio.rs`. Only `audio.rs`
//! deals with audio backends. This means that the whole of the code is blissfully
//! ignorant of the audio backend in use.
//!
//! `audio.rs` is also in charge of emitting signals whenever a change happens.
//! This means that PNMixer-rs design is quite signal-oriented, so to say.
//!
//! The ui code is nothing fancy. Each ui element...
//!
//! * is defined in a single file
//! * strives to be standalone
//! * accesses the sound system with function calls
//! * listens to signals from the audio subsystem to update its appearance
//!
//! There's something you should keep in mind. Audio on a computer is a shared
//! resource. PNMixer-rs isn't the only one that can change it. At any moment the
//! audio volume may be modified by someone else, and we must update the ui
//! accordingly. So listening to changes from the audio subsystem (and therefore
//! having a signal-oriented design) is the most obvious solution to solve that
//! problem.


#![warn(missing_docs)]

#![feature(alloc_system)]
extern crate alloc_system;

pub extern crate flexi_logger;
#[macro_use]
extern crate log;

#[macro_use]
extern crate error_chain;

#[macro_use]
extern crate serde_derive;
extern crate toml;
extern crate serde;

extern crate alsa;
extern crate alsa_sys;
extern crate ffi;
extern crate gdk;
extern crate gdk_pixbuf;
extern crate gdk_pixbuf_sys;
extern crate gdk_sys;
extern crate gio;
extern crate glib;
extern crate glib_sys;
extern crate gobject_sys;
pub extern crate gtk;
extern crate gtk_sys;
extern crate libc;
extern crate png;
extern crate w_result;
extern crate which;
extern crate x11;
extern crate xdg;

#[cfg(feature = "notify")]
pub extern crate libnotify;

#[macro_use]
extern crate lazy_static;

#[macro_use]
pub mod errors;

#[macro_use]
pub mod glade_helpers;

pub mod alsa_backend;
pub mod app_state;
pub mod audio_frontend;
pub mod gdk_x11;
pub mod hotkey;
pub mod hotkeys;
pub mod prefs;
pub mod support_alsa;
pub mod support_audio;
pub mod support_cmd;
#[macro_use]
pub mod support_ui;
pub mod ui_entry;
pub mod ui_hotkey_dialog;
pub mod ui_popup_menu;
pub mod ui_popup_window;
pub mod ui_prefs_dialog;
pub mod ui_tray_icon;

#[cfg(feature = "notify")]
pub mod notif;
