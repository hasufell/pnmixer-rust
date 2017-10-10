//! Global application state.


use audio::alsa::backend::*;
use audio::pulseaudio::*;
use audio::frontend::*;
use errors::*;
use gtk;
use hotkeys::Hotkeys;
use prefs::*;
use std::cell::RefCell;
use std::rc::Rc;
use support::audio::*;
use ui::entry::Gui;

#[cfg(feature = "notify")]
use notif::*;



// TODO: destructors
/// The global application state struct.
pub struct AppS<T>
where
    T: AudioFrontend,
{
    _cant_construct: (),
    /// Mostly static GUI state.
    pub gui: Gui,
    /// Audio state.
    pub audio: Rc<T>,
    /// Preferences state.
    pub prefs: RefCell<Prefs>,
    #[cfg(feature = "notify")]
    /// Notification state. In case of initialization failure, this
    /// is set to `None`.
    pub notif: Option<Notif>,
    /// Hotkey state.
    pub hotkeys: RefCell<Box<Hotkeys<T>>>, // Gets an Rc to Audio.
}


/// Create a new application state using the `AlsaBackend`.
pub fn new_alsa_appstate() -> AppS<AlsaBackend> {
    let prefs = RefCell::new(unwrap_error!(Prefs::new(), None));

    let card_name = prefs.borrow().device_prefs.card.clone();
    let chan_name = prefs.borrow().device_prefs.channel.clone();
    let audio = Rc::new(unwrap_error!(
        AlsaBackend::new(Some(card_name), Some(chan_name)),
        None
    ));
    return AppS::new(prefs, audio);
}

/// Create a new application state using the `PABackend`.
pub fn new_pa_appstate() -> AppS<PABackend> {
    let prefs = RefCell::new(unwrap_error!(Prefs::new(), None));

    let card_name = prefs.borrow().device_prefs.card.clone();
    let chan_name = prefs.borrow().device_prefs.channel.clone();
    let audio = Rc::new(unwrap_error!(
        PABackend::new(Some(card_name), Some(chan_name)),
        None
    ));
    return AppS::new(prefs, audio);
}


impl<T> AppS<T>
where
    T: AudioFrontend,
{
    /// Create an application state instance. There should really only be one.
    pub fn new(prefs: RefCell<Prefs>, audio: Rc<T>) -> Self {
        let builder_popup_window =
            gtk::Builder::new_from_string(include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/data/ui/popup-window.glade"
            )));
        let builder_popup_menu =
            gtk::Builder::new_from_string(include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/data/ui/popup-menu.glade"
            )));


        // TODO: better error handling
        #[cfg(feature = "notify")]
        let notif = result_warn!(Notif::new(&prefs.borrow()), None).ok();

        let hotkeys =
            unwrap_error!(
                wresult_warn!(Hotkeys::new(&prefs.borrow(), audio.clone()), None),
                None
            );

        let gui =
            Gui::new(builder_popup_window, builder_popup_menu, &prefs.borrow());

        return AppS {
            _cant_construct: (),
            gui,
            audio,
            prefs,
            #[cfg(feature = "notify")]
            notif,
            hotkeys: RefCell::new(hotkeys),
        };
    }


    /* some functions that need to be easily accessible */

    /// Update the tray icon state.
    pub fn update_tray_icon(&self) -> Result<()> {
        debug!("Update tray icon!");
        return self.gui.tray_icon.update_all(
            &self.prefs.borrow(),
            self.audio.as_ref(),
            None,
        );
    }

    /// Update the Popup Window state.
    pub fn update_popup_window(&self) -> Result<()> {
        debug!("Update PopupWindow!");
        return self.gui.popup_window.update(self.audio.as_ref());
    }

    #[cfg(feature = "notify")]
    /// Update the notification state.
    pub fn update_notify(&self) {
        match self.notif {
            Some(ref n) => n.reload(&self.prefs.borrow()),
            None => {
                warn!("Notification system not unitialized, skipping update")
            }
        }
    }

    #[cfg(not(feature = "notify"))]
    /// Update the notification state.
    pub fn update_notify(&self) {}

    /// Update the audio state.
    pub fn update_audio(&self, user: AudioUser) -> Result<()> {
        return audio_reload(self.audio.as_ref(), &self.prefs.borrow(), user);
    }

    /// Update the config file.
    pub fn update_config(&self) -> Result<()> {
        let prefs = self.prefs.borrow_mut();
        return prefs.store_config();
    }

    /// Update hotkey state.
    pub fn update_hotkeys(&self) -> Result<()> {
        let prefs = self.prefs.borrow();
        return self.hotkeys.borrow_mut().reload(&prefs);
    }
}
