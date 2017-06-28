extern crate gtk;
extern crate gtk_sys;
extern crate gdk;
extern crate gdk_sys;
extern crate glib;
extern crate ffi;
extern crate libc;


use errors::*;
use gtk::prelude::*;
use gdk::DeviceExt;
use gdk::{GrabOwnership, GrabStatus, BUTTON_PRESS_MASK, KEY_PRESS_MASK};
use gdk_sys::GDK_CURRENT_TIME;

pub fn set_slider(vol_scale_adj: &gtk::Adjustment, scale: f64) {
    vol_scale_adj.set_value(scale);
}

pub fn grab_devices(window: &gtk::Window) -> Result<()> {
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
