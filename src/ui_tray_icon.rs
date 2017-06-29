use app_state::*;
use gtk::prelude::*;
use gtk;



pub fn init_tray_icon(appstate: &AppS) {

    let ref tray_icon = appstate.status_icon;

    let popup_window: gtk::Window =
        appstate.builder_popup.get_object("popup_window").unwrap();

    tray_icon.connect_activate(move |_| if popup_window.get_visible() {
                                   popup_window.hide();
                               } else {
                                   popup_window.show_now();
                               });
    tray_icon.set_visible(true);
}
