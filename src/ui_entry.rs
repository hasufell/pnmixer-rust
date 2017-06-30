use app_state::*;
use audio::{AlsaCard, AudioSignal, AudioUser};
use std::cell::RefCell;
use std::rc::Rc;
use ui_popup_window::*;
use ui_tray_icon::*;



pub fn init(appstate: Rc<AppS>) {
    {
        let apps = appstate.clone();
        appstate.acard.borrow().connect_handler(Box::new(move |a, s, u| {
            match (s, u) {
                (AudioSignal::AudioValuesChanged,
                 AudioUser::AudioUserUnknown) => {
                    println!("External volume change!");

                }
                _ => println!("Nix"),
            }
        }));
    }

    init_tray_icon(appstate.clone());
    init_popup_window(appstate.clone());
}
