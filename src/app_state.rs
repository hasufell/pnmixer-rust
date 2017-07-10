use audio::Audio;
use errors::*;
use gtk;
use prefs::*;
use std::cell::RefCell;
use ui_entry::Gui;

#[cfg(feature = "notify")]
use notif::*;

// TODO: notify popups


// TODO: destructors

// TODO: glade stuff, config, alsacard
pub struct AppS {
    _cant_construct: (),
    pub gui: Gui,
    pub audio: Audio,
    pub prefs: RefCell<Prefs>,
    #[cfg(feature = "notify")]
    pub notif: Notif,
}


impl AppS {
    pub fn new() -> AppS {
        let builder_popup_window =
            gtk::Builder::new_from_string(include_str!(concat!(env!("CARGO_MANIFEST_DIR"),
                                                               "/data/ui/popup-window.glade")));
        let builder_popup_menu =
            gtk::Builder::new_from_string(include_str!(concat!(env!("CARGO_MANIFEST_DIR"),
                                                               "/data/ui/popup-menu.glade")));
        let prefs = RefCell::new(Prefs::new().unwrap());
        let gui =
            Gui::new(builder_popup_window, builder_popup_menu, &prefs.borrow());

        let card_name = prefs.borrow()
            .device_prefs
            .card
            .clone();
        let chan_name = prefs.borrow()
            .device_prefs
            .channel
            .clone();
        // TODO: better error handling
        #[cfg(feature = "notify")]
        let notif = Notif::new(&prefs.borrow()).unwrap();

        return AppS {
                   _cant_construct: (),
                   gui,
                   audio: Audio::new(Some(card_name), Some(chan_name)).unwrap(),
                   prefs,
                   #[cfg(feature = "notify")]
                   notif,
               };
    }


    /* some functions that need to be easily accessible */

    pub fn update_tray_icon(&self) -> Result<()> {
        debug!("Update tray icon!");
        return self.gui.tray_icon.update_all(&self.prefs.borrow(),
                                             &self.audio,
                                             None);
    }

    pub fn update_popup_window(&self) -> Result<()> {
        debug!("Update PopupWindow!");
        return self.gui.popup_window.update(&self.audio);
    }
}
