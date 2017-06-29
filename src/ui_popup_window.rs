use app_state::*;
use audio::AlsaCard;
use errors::*;
use gdk::DeviceExt;
use gdk::{GrabOwnership, GrabStatus, BUTTON_PRESS_MASK, KEY_PRESS_MASK};
use gdk;
use gdk_sys::{GDK_KEY_Escape, GDK_CURRENT_TIME};
use gtk::prelude::*;
use gtk;
use std::cell::RefCell;
use std::rc::Rc;



pub fn init_popup_window(appstate: &AppS, rc_acard: Rc<RefCell<AlsaCard>>) {
    /* popup_window.connect_show */
    {
        let popup_window: gtk::Window =
            appstate.builder_popup.get_object("popup_window").unwrap();
        let vol_scale_adj: gtk::Adjustment =
            appstate.builder_popup.get_object("vol_scale_adj").unwrap();
        let mute_check: gtk::CheckButton =
            appstate.builder_popup.get_object("mute_check").unwrap();
        let vol_scale: gtk::Scale =
            appstate.builder_popup.get_object("vol_scale").unwrap();

        let card = rc_acard.clone();
        popup_window.connect_show(move |w| {
            let acard = card.borrow();

            let cur_vol = try_w!(acard.vol());
            println!("Cur vol: {}", cur_vol);
            set_slider(&vol_scale_adj, cur_vol);

            let muted = acard.get_mute();
            update_mute_check(&mute_check, muted);

            vol_scale.grab_focus();
            try_w!(grab_devices(w));
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


fn set_slider(vol_scale_adj: &gtk::Adjustment, scale: f64) {
    vol_scale_adj.set_value(scale);
}


fn grab_devices(window: &gtk::Window) -> Result<()> {
    let device = gtk::get_current_event_device().ok_or("No current device")?;

    let gdk_window = window.get_window().ok_or("No window?!")?;

    /* Grab the mouse */
    let m_grab_status = device.grab(
        &gdk_window,
        GrabOwnership::None,
        true,
        BUTTON_PRESS_MASK,
        None,
        GDK_CURRENT_TIME as u32,
    );

    if m_grab_status != GrabStatus::Success {
        warn!(
            "Could not grab {}",
            device.get_name().unwrap_or(String::from("UNKNOWN DEVICE"))
        );
    }

    /* Grab the keyboard */
    let k_dev = device.get_associated_device().ok_or(
        "Couldn't get associated device",
    )?;

    let k_grab_status = k_dev.grab(
        &gdk_window,
        GrabOwnership::None,
        true,
        KEY_PRESS_MASK,
        None,
        GDK_CURRENT_TIME as u32,
    );
    if k_grab_status != GrabStatus::Success {
        warn!(
            "Could not grab {}",
            k_dev.get_name().unwrap_or(String::from("UNKNOWN DEVICE"))
        );
    }

    return Ok(());
}

