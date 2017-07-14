#![warn(missing_docs)]

#![feature(alloc_system)]
extern crate alloc_system;

pub extern crate flexi_logger;
#[macro_use]
pub extern crate log;

#[macro_use]
pub extern crate error_chain;

#[macro_use]
pub extern crate serde_derive;
pub extern crate toml;
pub extern crate serde;

pub extern crate alsa;
pub extern crate alsa_sys;
pub extern crate ffi;
pub extern crate gdk;
pub extern crate gdk_pixbuf;
pub extern crate gdk_pixbuf_sys;
pub extern crate gdk_sys;
pub extern crate gio;
pub extern crate glib;
pub extern crate glib_sys;
pub extern crate gobject_sys;
pub extern crate gtk;
pub extern crate gtk_sys;
pub extern crate libc;
pub extern crate png;
pub extern crate which;
pub extern crate xdg;

#[cfg(feature = "notify")]
pub extern crate libnotify;

#[macro_use]
pub mod errors;

#[macro_use]
pub mod glade_helpers;

pub mod alsa_card;
pub mod app_state;
pub mod audio;
pub mod prefs;
pub mod support_alsa;
pub mod support_audio;
pub mod support_cmd;
#[macro_use]
pub mod support_ui;
pub mod ui_entry;
pub mod ui_popup_menu;
pub mod ui_popup_window;
pub mod ui_prefs_dialog;
pub mod ui_tray_icon;

#[cfg(feature = "notify")]
pub mod notif;

