use app_state::*;
use audio::AudioUser::*;
use errors::*;
use gdk::DeviceExt;
use gdk::{GrabOwnership, GrabStatus, BUTTON_PRESS_MASK, KEY_PRESS_MASK};
use gdk;
use gdk_sys::{GDK_KEY_Escape, GDK_CURRENT_TIME};
use glib;
use gtk::prelude::*;
use gtk;
use std::rc::Rc;



pub fn init_popup_window(appstate: Rc<AppS>) {
    let mut toggle_signal = 0;

    /* mute_check.connect_toggled */
    {
        let _appstate = appstate.clone();
        let mute_check = &appstate.clone()
                              .gui
                              .popup_window
                              .mute_check;
        toggle_signal =
            mute_check.connect_toggled(move |_| {
                                           on_mute_check_toggled(&_appstate)
                                       });
    }

    /* popup_window.connect_show */
    {
        let _appstate = appstate.clone();
        let popup_window = &appstate.clone()
                                .gui
                                .popup_window
                                .window;
        popup_window.connect_show(move |w| {
                                      on_popup_window_show(w,
                                                           &_appstate,
                                                           toggle_signal)
                                  });
    }

    /* vol_scale_adj.connect_value_changed */
    {
        let _appstate = appstate.clone();
        let vol_scale_adj = &appstate.clone()
                                 .gui
                                 .popup_window
                                 .vol_scale_adj;
        vol_scale_adj.connect_value_changed(
            move |_| on_vol_scale_value_changed(&_appstate),
        );
    }

    /* popup_window.connect_event */
    {
        let _appstate = appstate.clone();
        let popup_window = &appstate.clone()
                                .gui
                                .popup_window
                                .window;
        popup_window.connect_event(move |w, e| {
                                       on_popup_window_event(w, e, &_appstate)
                                   });
    }
}


fn on_popup_window_show(window: &gtk::Window,
                        appstate: &AppS,
                        toggle_signal: u64) {
    let acard = appstate.acard.borrow();
    let popup_window = &appstate.gui.popup_window;

    let cur_vol = try_w!(acard.vol());
    set_slider(&popup_window.vol_scale_adj, cur_vol);

    let muted = acard.get_mute();
    update_mute_check(&appstate, toggle_signal, muted);

    popup_window.vol_scale.grab_focus();
    try_w!(grab_devices(window));
}


fn on_popup_window_event(w: &gtk::Window,
                         e: &gdk::Event,
                         appstate: &AppS)
                         -> gtk::Inhibit {
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
}


fn on_vol_scale_value_changed(appstate: &AppS) {
    let acard = appstate.acard.borrow();

    let val = appstate.gui
        .popup_window
        .vol_scale
        .get_value();

    try_w!(acard.set_vol(val, AudioUserPopup));
}


fn on_mute_check_toggled(appstate: &AppS) {
    let acard = appstate.acard.borrow();

    let muted = try_w!(acard.get_mute());
    let _ = try_w!(acard.set_mute(!muted, AudioUserPopup));
}


pub fn update_mute_check(appstate: &AppS,
                         toggle_signal: u64,
                         muted: Result<bool>) {
    let check_button = &appstate.gui.popup_window.mute_check;

    glib::signal_handler_block(check_button, toggle_signal);

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

    glib::signal_handler_unblock(check_button, toggle_signal);
}


pub fn set_slider(vol_scale_adj: &gtk::Adjustment, scale: f64) {
    vol_scale_adj.set_value(scale);
}


fn grab_devices(window: &gtk::Window) -> Result<()> {
    let device = gtk::get_current_event_device().ok_or("No current device")?;

    let gdk_window = window.get_window().ok_or("No window?!")?;

    /* Grab the mouse */
    let m_grab_status =
        device.grab(&gdk_window,
                    GrabOwnership::None,
                    true,
                    BUTTON_PRESS_MASK,
                    None,
                    GDK_CURRENT_TIME as u32);

    if m_grab_status != GrabStatus::Success {
        warn!("Could not grab {}",
              device.get_name().unwrap_or(String::from("UNKNOWN DEVICE")));
    }

    /* Grab the keyboard */
    let k_dev = device.get_associated_device()
        .ok_or("Couldn't get associated device")?;

    let k_grab_status = k_dev.grab(&gdk_window,
                                   GrabOwnership::None,
                                   true,
                                   KEY_PRESS_MASK,
                                   None,
                                   GDK_CURRENT_TIME as u32);
    if k_grab_status != GrabStatus::Success {
        warn!("Could not grab {}",
              k_dev.get_name().unwrap_or(String::from("UNKNOWN DEVICE")));
    }

    return Ok(());
}
