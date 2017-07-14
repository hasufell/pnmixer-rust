//! The notification subsystem.
//!
//! This subsystem utilizes libnotify to send notifications as popups
//! to the desktop.


use app_state::*;
use audio::*;
use errors::*;
use glib::prelude::*;
use libnotify;
use prefs::*;
use std::cell::Cell;
use std::rc::Rc;



/// An expression of our notification system. Holds all the relevant information
/// needed by Gtk+ callbacks to interact with libnotify.
pub struct Notif {
    enabled: Cell<bool>,
    from_popup: Cell<bool>,
    from_tray: Cell<bool>,
    // TODO: from hotkey
    from_external: Cell<bool>,

    volume_notif: libnotify::Notification,
    text_notif: libnotify::Notification,
}

impl Notif {
    /// Create a new notification instance from the current preferences.
    pub fn new(prefs: &Prefs) -> Result<Self> {
        let notif = Notif {
            enabled: Cell::new(false),
            from_popup: Cell::new(false),
            from_tray: Cell::new(false),
            from_external: Cell::new(false),

            volume_notif: libnotify::Notification::new("", None, None),
            text_notif: libnotify::Notification::new("", None, None),
        };

        notif.reload(prefs)?;

        return Ok(notif);
    }

    /// Reload the notification instance from the current
    /// preferences.
    pub fn reload(&self, prefs: &Prefs) -> Result<()> {
        let timeout = prefs.notify_prefs.notifcation_timeout;

        self.enabled.set(prefs.notify_prefs.enable_notifications);
        self.from_popup.set(prefs.notify_prefs.notify_popup);
        self.from_tray.set(prefs.notify_prefs.notify_mouse_scroll);
        self.from_external.set(prefs.notify_prefs.notify_external);

        self.volume_notif.set_timeout(timeout as i32);
        self.volume_notif.set_hint("x-canonical-private-synchronous",
                                   Some("".to_variant()));


        self.text_notif.set_timeout(timeout as i32);
        self.text_notif.set_hint("x-canonical-private-synchronous",
                                 Some("".to_variant()));

        return Ok(());
    }

    /// Shows a volume notification, e.g. for volume or mute state change.
    pub fn show_volume_notif(&self, audio: &Audio) -> Result<()> {
        let vol = audio.vol()?;
        let vol_level = audio.vol_level();

        let icon = {
            match vol_level {
                VolLevel::Muted => "audio-volume-muted",
                VolLevel::Off => "audio-volume-off",
                VolLevel::Low => "audio-volume-low",
                VolLevel::Medium => "audio-volume-medium",
                VolLevel::High => "audio-volume-high",
            }
        };

        let summary = {
            match vol_level {
                VolLevel::Muted => String::from("Volume muted"),
                _ => {
                    format!("{} ({})\nVolume: {}",
                            audio.acard
                                .borrow()
                                .card_name()?,
                            audio.acard
                                .borrow()
                                .chan_name()?,
                            vol as i32)
                }
            }
        };

        // TODO: error handling
        self.volume_notif.update(summary.as_str(), None, Some(icon)).unwrap();
        self.volume_notif.set_hint("value", Some((vol as i32).to_variant()));
        // TODO: error handling
        self.volume_notif.show().unwrap();

        return Ok(());
    }


    /// Shows a text notification, e.g. for warnings or errors.
    pub fn show_text_notif(&self, summary: &str, body: &str) -> Result<()> {
        // TODO: error handling
        self.text_notif.update(summary, Some(body), None).unwrap();
        // TODO: error handling
        self.text_notif.show().unwrap();

        return Ok(());
    }
}



/// Initialize the notification subsystem.
pub fn init_notify(appstate: Rc<AppS>) {
    debug!("Blah");
    {
        /* connect handler */
        let apps = appstate.clone();
        appstate.audio.connect_handler(Box::new(move |s, u| {
            let notif = &apps.notif;
            if !notif.enabled.get() {
                return;
            }
            match (s,
                   u,
                   (notif.from_popup.get(),
                    notif.from_tray.get(),
                    notif.from_external.get())) {
                (AudioSignal::NoCard, _, _) => try_w!(notif.show_text_notif("No sound card", "No playable soundcard found")),
                (AudioSignal::CardDisconnected, _, _) => try_w!(notif.show_text_notif("Soundcard disconnected", "Soundcard has been disconnected, reloading sound system...")),
                (AudioSignal::CardError, _, _) => (),
                (AudioSignal::ValuesChanged,
                 AudioUser::TrayIcon,
                 (_, true, _)) => try_w!(notif.show_volume_notif(&apps.audio)),
                (AudioSignal::ValuesChanged,
                 AudioUser::Popup,
                 (true, _, _)) => try_w!(notif.show_volume_notif(&apps.audio)),
                (AudioSignal::ValuesChanged,
                 AudioUser::Unknown,
                 (_, _, true)) => try_w!(notif.show_volume_notif(&apps.audio)),

                 // TODO hotkeys
                _ => (),
            }
        }));

    }
}
