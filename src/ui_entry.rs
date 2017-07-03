use app_state::*;
use audio::*;
use std::rc::Rc;
use ui_popup_menu::*;
use ui_popup_window::*;
use ui_tray_icon::*;
use ui_prefs_dialog::*;



pub fn init(appstate: Rc<AppS>) {
    {
        let apps = appstate.clone();
        appstate.audio.connect_handler(Box::new(move |s, u| match (s, u) {
                                                    (AudioSignal::ValuesChanged,
                 AudioUser::Unknown) => {
                    debug!("External volume change!");

                }
                                                    _ => debug!("Nix"),
                                                }));
    }

    init_tray_icon(appstate.clone());
    init_popup_window(appstate.clone());
    init_popup_menu(appstate.clone());
}
