use gtk;
use audio::AlsaCard;
use std::cell::RefCell;
use std::rc::Rc;


// TODO: destructors

// TODO: glade stuff, config, alsacard
pub struct AppS {
    pub gui: Gui,
    pub acard: Rc<RefCell<AlsaCard>>,
}


impl AppS {
    pub fn new() -> AppS {
        let builder_popup = gtk::Builder::new_from_string(include_str!(
            "../data/ui/popup-window-vertical.glade"
        ));
        return AppS {
            gui: Gui::new(builder_popup),
            acard: Rc::new(RefCell::new(
                AlsaCard::new(None, Some(String::from("Master"))).unwrap(),
            )),
        };
    }
}


pub struct Gui {
    pub status_icon: gtk::StatusIcon,
    pub popup_window: PopupWindow,
}


impl Gui {
    pub fn new(builder: gtk::Builder) -> Gui {
        return Gui {
            status_icon: gtk::StatusIcon::new_from_icon_name("pnmixer"),
            popup_window: PopupWindow::new(builder),
        };
    }
}


pub struct PopupWindow {
    pub window: gtk::Window,
    pub vol_scale_adj: gtk::Adjustment,
    pub vol_scale: gtk::Scale,
    pub mute_check: gtk::CheckButton,
}


impl PopupWindow {
    pub fn new(builder: gtk::Builder) -> PopupWindow {
        return PopupWindow {
            window: builder.get_object("popup_window").unwrap(),
            vol_scale_adj: builder.get_object("vol_scale_adj").unwrap(),
            vol_scale: builder.get_object("vol_scale").unwrap(),
            mute_check: builder.get_object("mute_check").unwrap(),
        };
    }
}
