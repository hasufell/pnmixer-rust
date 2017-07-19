#![feature(alloc_system)]
extern crate alloc_system;

extern crate pnmixerlib;

use pnmixerlib::*;

use app_state::*;
use std::rc::Rc;


fn main() {
    gtk::init().unwrap();

    flexi_logger::LogOptions::new()
       .log_to_file(false)
       // ... your configuration options go here ...
       .init(Some("pnmixer=debug".to_string()))
       .unwrap_or_else(|e| panic!("Logger initialization failed with {}", e));

    let apps = Rc::new(new_alsa_appstate());

    ui::entry::init(apps);

    gtk::main();

    // TODO: clean deallocation?
}
