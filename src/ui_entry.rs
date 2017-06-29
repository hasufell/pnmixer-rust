use app_state::*;
use audio::AlsaCard;
use std::cell::RefCell;
use std::rc::Rc;
use ui_popup_window::*;
use ui_tray_icon::*;



pub fn init<'a>(appstate: &'a AppS, rc_acard: Rc<RefCell<AlsaCard>>) {

    init_tray_icon(&appstate);
    init_popup_window(&appstate, rc_acard);
}
