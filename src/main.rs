extern crate gtk;
extern crate gdk;
extern crate alsa;

// #[macro_use]
// extern crate lazy_static;

// use gdk::EventButton;
// use gdk::EventType;


// use std::ops::Deref;

// use std::boxed::Box;
// use std::rc::Rc;
// use std::sync::Arc;

use gtk::prelude::*;

use alsa::mixer::SelemChannelId::*;

mod audio;


fn main() {
    gtk::init().unwrap();

    let tray_icon = gtk::StatusIcon::new_from_icon_name("pnmixer");

    let glade_src = include_str!("../data/ui/popup-window-vertical.glade");
    let builder_popup = gtk::Builder::new_from_string(glade_src);

    let popup_window: gtk::Window = builder_popup.get_object("popup_window").unwrap();

    // let builder_prefs_dialog = Rc::new(Builder::new_from_file("data/ui/prefs-dialog.glade"));

    // with_builder!(builder_popup_window,
                 // tray_icon.connect_activate(move |_| {
                     // let popup_window: gtk::Window = builder
                         // .get_object("popup_window").unwrap();
                     // popup_window.show_all();
                 // })
             // );

    // 2nd binding

    tray_icon.connect_button_press_event(move |_, e| {
        let bt = e.as_ref().button;
        match bt {
            3 => {
                popup_window.show_all();
                return true;
            }
            _ => {
                println!("Blah");
                return false;
            }
        }
    });

    let alsa_card = audio::get_default_alsa_card();
    let mixer = audio::get_mixer(alsa_card);
    let channel = audio::get_channel_by_name(&mixer, String::from("Master")).unwrap();

    println!("Range: {:?}", channel.get_playback_volume_range());
    println!("Channel: {}", channel.get_playback_volume(FrontCenter).unwrap());

    tray_icon.set_visible(true);

    gtk::main();
}
