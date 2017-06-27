extern crate gtk;
extern crate gtk_sys;
extern crate gdk;
extern crate gdk_sys;
extern crate alsa;
extern crate std;

use gtk::prelude::*;
use gdk_sys::GDK_KEY_Escape;

use gui;
use audio;
use app_state::*;
use errors::*;


pub fn init<'a>(appstate: &'a AppS) {

    init_tray_icon(&appstate);
    init_popup_window(&appstate);
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

fn init_popup_window(appstate: &AppS) {
    /* popup_window.connect_show */
    {
        let popup_window: gtk::Window =
            appstate.builder_popup.get_object("popup_window").unwrap();
        let vol_scale_adj: gtk::Adjustment =
            appstate.builder_popup.get_object("vol_scale_adj").unwrap();
        let mute_check: gtk::CheckButton =
            appstate.builder_popup.get_object("mute_check").unwrap();

        popup_window.connect_show(move |_| {
            let alsa_card = audio::get_default_alsa_card();
            let mixer = try_w!(audio::get_mixer(&alsa_card));
            let selem = try_w!(audio::get_selem_by_name(
                &mixer,
                String::from("Master"),
            ));
            let cur_vol = try_w!(audio::get_vol(&selem));
            gui::set_slider(&vol_scale_adj, cur_vol);

            let muted = audio::get_mute(&selem);
            update_mute_check(&mute_check, muted);
        });
    }

    /* mute_check.connect_toggled */
    {
        let mute_check: gtk::CheckButton =
            appstate.builder_popup.get_object("mute_check").unwrap();

        mute_check.connect_toggled(move |_| {
            let alsa_card = audio::get_default_alsa_card();
            let mixer = try_w!(audio::get_mixer(&alsa_card));
            let selem = try_w!(audio::get_selem_by_name(
                &mixer,
                String::from("Master"),
            ));

            let muted = try_w!(audio::get_mute(&selem));
            let _ = try_w!(audio::set_mute(&selem, !muted));
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
        },
        Err(_) => {
            /* can't figure out whether channel is muted, grey out */
            check_button.set_active(true);
            check_button.set_sensitive(false);
            check_button.set_tooltip_text("Soundcard has no mute switch");
        }
    }
}

