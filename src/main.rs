extern crate flexi_logger;
#[macro_use]
extern crate log;

extern crate gtk;
extern crate gtk_sys;
extern crate gdk;
extern crate gdk_sys;
extern crate alsa;

// use std::ops::Deref;

// use std::boxed::Box;
// use std::rc::Rc;
// use std::sync::Arc;

use gtk::prelude::*;


use gdk_sys::GDK_KEY_Escape;


mod audio;
mod gui;


fn main() {
    gtk::init().unwrap();

    flexi_logger::LogOptions::new()
       .log_to_file(false)
       // ... your configuration options go here ...
       .init(Some("info".to_string()))
       .unwrap_or_else(|e| panic!("Logger initialization failed with {}", e));

    let tray_icon = gtk::StatusIcon::new_from_icon_name("pnmixer");

    let glade_src = include_str!("../data/ui/popup-window-vertical.glade");
    let builder_popup = gtk::Builder::new_from_string(glade_src);

    {
        let popup_window: gtk::Window = builder_popup.get_object("popup_window")
            .unwrap();
        let vol_scale: gtk::Scale = builder_popup.get_object("vol_scale")
            .unwrap();

        tray_icon.connect_activate(move |_| if popup_window.get_visible() {
                                       popup_window.hide();
                                   } else {
                                       popup_window.show_now();
                                       vol_scale.grab_focus();
                                       gui::grab_devices(&popup_window);
                                   });
    }
    {
        let popup_window: gtk::Window = builder_popup.get_object("popup_window")
            .unwrap();
        let vol_scale_adj: gtk::Adjustment = builder_popup.get_object("vol_scale_adj")
            .unwrap();
        popup_window.connect_show(move |_| {
            let alsa_card = audio::get_default_alsa_card();
            let mixer = audio::get_mixer(alsa_card);
            let selem = audio::get_selem_by_name(&mixer,
                                                 String::from("Master"))
                    .unwrap();
            gui::set_slider(&vol_scale_adj, audio::get_vol(selem).unwrap())
        });
    }

    {
        let popup_window: gtk::Window = builder_popup.get_object("popup_window")
            .unwrap();
        popup_window.connect_event(move |w, e| {
            match gdk::Event::get_event_type(e) {
                gdk::EventType::GrabBroken => w.hide(),
                gdk::EventType::KeyPress => {
                    let key: gdk::EventKey = e.clone().downcast().unwrap();
                    if key.get_keyval() == (GDK_KEY_Escape as u32) {
                        w.hide();
                    }
                }
                gdk::EventType::ButtonPress => {
                    let device = gtk::get_current_event_device().unwrap();
                    let (window, _, _) =
                        gdk::DeviceExt::get_window_at_position(&device);
                    if window.is_none() {
                        w.hide();
                    }
                }
                _ => (),
            }

            return Inhibit(false);
        });
    }

    tray_icon.set_visible(true);

    gtk::main();
}
