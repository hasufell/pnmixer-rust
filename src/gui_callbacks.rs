extern crate gtk;
extern crate gtk_sys;
extern crate gdk;
extern crate gdk_sys;
extern crate alsa;
extern crate std;

use gtk::prelude::*;
use gdk_sys::GDK_KEY_Escape;

use gui;
use app_state::*;
use errors::*;
use std::cell::RefCell;
use std::rc::Rc;


pub fn init<'a>(appstate: &'a AppS, rc_acard: Rc<RefCell<AlsaCard>>) {

    init_tray_icon(&appstate);
    init_popup_window(&appstate, rc_acard);
}


fn init_tray_icon(appstate: &AppS) {

    let ref tray_icon = appstate.status_icon;

    let popup_window: gtk::Window =
        appstate.builder_popup.get_object("popup_window").unwrap();
    let vol_scale: gtk::Scale =
        appstate.builder_popup.get_object("vol_scale").unwrap();

    tray_icon.connect_activate(move |_| if popup_window.get_visible() {
        popup_window.hide();
    } else {
        popup_window.show_now();
        vol_scale.grab_focus();
        try_w!(gui::grab_devices(&popup_window));
    });
    tray_icon.set_visible(true);
}

fn init_popup_window(appstate: &AppS, rc_acard: Rc<RefCell<AlsaCard>>) {
    /* popup_window.connect_show */
    {
        let popup_window: gtk::Window =
            appstate.builder_popup.get_object("popup_window").unwrap();
        let vol_scale_adj: gtk::Adjustment =
            appstate.builder_popup.get_object("vol_scale_adj").unwrap();
        let mute_check: gtk::CheckButton =
            appstate.builder_popup.get_object("mute_check").unwrap();

        let card = rc_acard.clone();
        popup_window.connect_show(move |_| {
            let acard = card.borrow();

            let cur_vol = try_w!(acard.vol());
            gui::set_slider(&vol_scale_adj, cur_vol);

            let muted = acard.get_mute();
            update_mute_check(&mute_check, muted);
        });
    }

    /* vol_scale_adj.connect_value_changed */
    {
        let vol_scale_adj: Rc<gtk::Adjustment> =
            Rc::new(
                appstate.builder_popup.get_object("vol_scale_adj").unwrap(),
            );

        let card = rc_acard.clone();
        let vol_scale = vol_scale_adj.clone();
        vol_scale_adj.connect_value_changed(move |_| {
            let acard = card.borrow();
            let val = vol_scale.get_value();

            try_w!(acard.set_vol(val));
        });
    }

    /* mute_check.connect_toggled */
    {
        let mute_check: gtk::CheckButton =
            appstate.builder_popup.get_object("mute_check").unwrap();

        let card = rc_acard.clone();
        mute_check.connect_toggled(move |_| {
            let acard = card.borrow();

            let muted = try_w!(acard.get_mute());
            let _ = try_w!(acard.set_mute(!muted));
        });
    }

    /* popup_window.connect_event */
    {
        let popup_window: gtk::Window =
            appstate.builder_popup.get_object("popup_window").unwrap();
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
                    let device = try_wr!(
                        gtk::get_current_event_device().ok_or(
                            "No current event device!",
                        ),
                        Inhibit(false)
                    );
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
}

fn update_mute_check(check_button: &gtk::CheckButton, muted: Result<bool>) {
    match muted {
        Ok(val) => {
            check_button.set_active(val);
            check_button.set_tooltip_text("");
        }
        Err(_) => {
            /* can't figure out whether channel is muted, grey out */
            check_button.set_active(true);
            check_button.set_sensitive(false);
            check_button.set_tooltip_text("Soundcard has no mute switch");
        }
    }
}
