extern crate flexi_logger;
#[macro_use]
extern crate log;

#[macro_use]
extern crate error_chain;

extern crate gtk;
extern crate gtk_sys;
extern crate gdk;
extern crate gdk_sys;
extern crate alsa;

use gtk::prelude::*;
use gdk_sys::GDK_KEY_Escape;
use app_state::*;
use std::cell::Cell;
use std::boxed::Box;


#[macro_use]
mod errors;

mod audio;
mod gui;
mod gui_callbacks;
mod app_state;


fn main() {
    gtk::init().unwrap();

    let ref apps = AppS {
        status_icon: gtk::StatusIcon::new_from_icon_name("pnmixer"),
        builder_popup: gtk::Builder::new_from_string(include_str!("../data/ui/popup-window-vertical.glade")),
    };

    let alsa_card = audio::get_default_alsa_card();
    let mixer = audio::get_mixer(&alsa_card).unwrap();
    let selem = audio::get_selem_by_name(
        &mixer,
        String::from("Master"),
    ).unwrap();

    let ref acard = AlsaCard {
        card: Cell::new(alsa_card),
        mixer: Cell::new(mixer),
        selem: Cell::new(selem),
    };

    flexi_logger::LogOptions::new()
       .log_to_file(false)
       // ... your configuration options go here ...
       .init(Some("info".to_string()))
       .unwrap_or_else(|e| panic!("Logger initialization failed with {}", e));


    gui_callbacks::init(apps);

    gtk::main();
}
