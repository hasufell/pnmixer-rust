use app_state::*;
use audio::{AlsaCard, AudioSignal, AudioUser};
use std::cell::RefCell;
use std::rc::Rc;
use ui_popup_window::*;
use ui_tray_icon::*;



pub fn init(appstate: Rc<AppS>) {
    let s1 = appstate.clone();
    let s2 = appstate.clone();

    appstate.acard.borrow().connect_handler(
        Box::new(|_, s, u| {
            println!("In der closure");
            match (s, u) {
            (AudioSignal::AudioValuesChanged, AudioUser::AudioUserUnknown) => {
                println!("Gaga");
            }
            _ => println!("Nix"),
        }}),
    );

    init_tray_icon(s1);
    init_popup_window(s2);
}
