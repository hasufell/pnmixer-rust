use app_state::*;
use gdk;
use gdk_pixbuf;
use gdk_sys;
use glib;
use glib_sys;
use std::mem;
use gtk::prelude::*;
use std::rc::Rc;
use std::cell::RefCell;
use libc;
use audio::AudioUser::*;
use errors::*;



// struct VolMeter {
    // pub red: u8,
    // pub green: u8,
    // pub blue: u8,
    // pub x_offset_pct: i64,
    // pub y_offset_pct: i64,
    // /* dynamic */
    // pub pixbuf: gdk_pixbuf::Pixbuf,
    // pub width: i64,
    // pub row: u8,
// }


// impl VolMeter {
    // pub fn new() -> VolMeter {
        // let pixbux = Pixbuf::new();
        // return VolMeter {
            // red: 255,
            // green: 255,
            // blue: 255,
            // x_offset_pct: 0,
            // y_offset_pct: 0,
            // pixbuf: ,
            // width: ,
            // row: ,
        // }
    // }
// }


fn pixbuf_new_from_stock(icon_name: String, size: u32) {

}



pub fn init_tray_icon(appstate: Rc<AppS>) {
    /* tray_icon.connect_activate */
    {
        let apps = appstate.clone();
        let tray_icon = &appstate.gui.status_icon;
        tray_icon.connect_activate(move |_| on_tray_icon_activate(&apps));
        tray_icon.set_visible(true);
    }

    /* tray_icon.connect_scroll_event */
    {
        let apps = appstate.clone();
        let tray_icon = &appstate.clone().gui.status_icon;
        tray_icon.connect_scroll_event(move |_, e| {
                                           on_tray_icon_scroll_event(&apps, &e)
                                       });
    }

    /* tray_icon.connect_popup_menu */
    {
        let apps = appstate.clone();
        let tray_icon = &appstate.clone().gui.status_icon;
        tray_icon.connect_popup_menu(move |_, _, _| {
                                         on_tray_icon_popup_menu(&apps)
                                     });
    }
}


fn on_tray_icon_activate(appstate: &AppS) {
    let popup_window = &appstate.gui.popup_window.popup_window;

    if popup_window.get_visible() {
        popup_window.hide();
    } else {
        popup_window.show_now();
    }
}


fn on_tray_icon_popup_menu(appstate: &AppS) {
    let popup_window = &appstate.gui.popup_window.popup_window;
    let popup_menu = &appstate.gui.popup_menu.menu;

    popup_window.hide();
    popup_menu.popup_at_pointer(None);
}


fn on_tray_icon_scroll_event(appstate: &AppS,
                             event: &gdk::EventScroll)
                             -> bool {

    let audio = &appstate.audio;

    let scroll_dir: gdk::ScrollDirection = event.get_direction();
    match scroll_dir {
        gdk::ScrollDirection::Up => {
            try_wr!(appstate.audio.increase_vol(AudioUserTrayIcon), false);
        }
        gdk::ScrollDirection::Down => {
            try_wr!(appstate.audio.decrease_vol(AudioUserTrayIcon), false);
        }
        _ => (),
    }

    return false;
}
