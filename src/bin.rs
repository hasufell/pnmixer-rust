#![feature(alloc_system)]
extern crate alloc_system;

extern crate pnmixerlib;

use pnmixerlib::*;

use app_state::*;
#[cfg(feature = "notify")]
use libnotify::*;
use std::rc::Rc;

fn main() {
    gtk::init().unwrap();

    // TODO: error handling
    #[cfg(feature = "notify")]
    init("PNMixer-rs").unwrap();

    flexi_logger::LogOptions::new()
       .log_to_file(false)
       // ... your configuration options go here ...
       .init(Some("pnmixer=debug".to_string()))
       .unwrap_or_else(|e| panic!("Logger initialization failed with {}", e));

    let apps = Rc::new(AppS::new());

    ui_entry::init(apps);

    gtk::main();

    #[cfg(feature = "notify")]
    uninit();
}