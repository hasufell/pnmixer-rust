#![feature(alloc_system)]
extern crate alloc_system;

extern crate flexi_logger;
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
extern crate glib;
extern crate glib_sys;
extern crate gobject_sys;
extern crate gtk;
extern crate gtk_sys;
extern crate libc;
extern crate which;
extern crate xdg;

use std::rc::Rc;

#[macro_use]
mod errors;

#[macro_use]
mod glade_helpers;

mod alsa_card;
mod app_state;
mod audio;
mod ui_entry;
mod ui_popup_menu;
mod ui_popup_window;
mod ui_prefs_dialog;
mod ui_tray_icon;
mod prefs;
mod support_alsa;
mod support_audio;
mod support_cmd;
mod support_ui;

use app_state::*;



fn main() {
    gtk::init().unwrap();

    flexi_logger::LogOptions::new()
       .log_to_file(false)
       // ... your configuration options go here ...
       .init(Some("pnmixer=debug".to_string()))
       .unwrap_or_else(|e| panic!("Logger initialization failed with {}", e));

    let apps = Rc::new(AppS::new());

    ui_entry::init(apps);

    gtk::main();
}
