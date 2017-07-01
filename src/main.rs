#![feature(alloc_system)]
extern crate alloc_system;

extern crate flexi_logger;
#[macro_use]
extern crate log;

#[macro_use]
extern crate error_chain;

extern crate alsa;
extern crate alsa_sys;
extern crate ffi;
extern crate gdk;
extern crate gdk_sys;
extern crate glib;
extern crate glib_sys;
extern crate gobject_sys;
extern crate gtk;
extern crate gtk_sys;
extern crate libc;

use std::rc::Rc;

#[macro_use]
mod errors;

#[macro_use]
mod glade_helpers;

mod app_state;
mod audio;
mod myalsa;
mod ui_entry;
mod ui_popup_menu;
mod ui_popup_window;
mod ui_tray_icon;

use app_state::*;



fn main() {
    gtk::init().unwrap();

    let apps = Rc::new(AppS::new());

    flexi_logger::LogOptions::new()
       .log_to_file(false)
       // ... your configuration options go here ...
       .init(Some("pnmixer=debug".to_string()))
       .unwrap_or_else(|e| panic!("Logger initialization failed with {}", e));

    ui_entry::init(apps);

    gtk::main();
}
