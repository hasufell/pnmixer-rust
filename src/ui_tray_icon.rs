use app_state::*;
use gtk::prelude::*;
use std::rc::Rc;



pub fn init_tray_icon(appstate: Rc<AppS>) {

    let tray_icon = &appstate.clone().gui.status_icon;
    tray_icon.connect_activate(
        move |_| on_tray_icon_activate(&appstate.clone()),
    );
    tray_icon.set_visible(true);
}


fn on_tray_icon_activate(appstate: &AppS) {
    let popup_window = &appstate.gui.popup_window.window;

    if popup_window.get_visible() {
        popup_window.hide();
    } else {
        popup_window.show_now();
    }
}
