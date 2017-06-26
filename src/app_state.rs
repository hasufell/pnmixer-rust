use audio::{Audio, AudioUser};
use errors::*;
use gtk;
use prefs::*;
use std::cell::RefCell;
use support_audio::*;
use ui_entry::Gui;

#[cfg(feature = "notify")]
use notif::*;



// TODO: destructors
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

    #[cfg(feature = "notify")]
    pub fn update_notify(&self) -> Result<()> {
        return self.notif.reload(&self.prefs.borrow());
    }

    #[cfg(not(feature = "notify"))]
    pub fn update_notify(&self) -> Result<()> {
        return Ok(());
    }

    pub fn update_audio(&self, user: AudioUser) -> Result<()> {
        return audio_reload(&self.audio, &self.prefs.borrow(), user);
    }

    pub fn update_config(&self) -> Result<()> {
        let prefs = self.prefs.borrow_mut();
        return prefs.store_config();
    }
}
