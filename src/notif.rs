use app_state::*;
use audio::*;
use gtk::DialogExt;
use gtk::MessageDialogExt;
use gtk::WidgetExt;
use gtk::WindowExt;
use gtk;
use gtk_sys::{GTK_DIALOG_DESTROY_WITH_PARENT, GTK_RESPONSE_YES};
use prefs::*;
use std::cell::Cell;
use std::cell::RefCell;
use std::rc::Rc;
use support_audio::*;
use support_ui::*;
use ui_popup_menu::*;
use ui_popup_window::*;
use ui_prefs_dialog::*;
use ui_tray_icon::*;
use errors::*;

use libnotify;
use std::thread;
use std::time::Duration;
use glib::Variant;
use glib::prelude::*;


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
    pub fn new(prefs: &Prefs) -> Result<Self> {
        let notif = Notif {
            enabled: Cell::new(false),
            from_popup: Cell::new(false),
            from_tray: Cell::new(false),
            from_external: Cell::new(false),

            volume_notif: libnotify::Notification::new("", None, None).unwrap(),
            text_notif: libnotify::Notification::new("", None, None).unwrap(),
        };

        notif.reload(prefs)?;

        return Ok(notif);
    }

    pub fn reload(&self, prefs: &Prefs) -> Result<()> {
        let timeout = prefs.notify_prefs.notifcation_timeout;

        self.enabled.set(prefs.notify_prefs.enable_notifications);
        self.enabled.set(prefs.notify_prefs.notify_popup);
        self.enabled.set(prefs.notify_prefs.notify_mouse_scroll);
        self.enabled.set(prefs.notify_prefs.notify_external);

        self.volume_notif
            .set_notification_timeout(timeout as i32);
        self.volume_notif
            .set_hint("x-canonical-private-synchronous", Some("".to_variant()))?;


        self.text_notif
            .set_notification_timeout(timeout as i32);
        self.text_notif
            .set_hint("x-canonical-private-synchronous", Some("".to_variant()))?;

        return Ok(());
    }

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
                            audio.acard.borrow().card_name()?,
                            audio.acard.borrow().chan_name()?,
                            vol)
                }
            }
        };

        self.volume_notif
            .update(summary.as_str(), None, Some(icon))?;
        self.volume_notif
            .set_hint("value", Some(vol.to_variant()))?;
        self.volume_notif.show()?;

        return Ok(());
    }


    pub fn show_text_notif(&self, summary: &str, body: &str) -> Result<()> {
        self.text_notif
            .update(summary, Some(body), None)?;
        self.text_notif.show()?;

        return Ok(());
    }
}



pub fn init_notify(appstate: Rc<AppS>) {
    {
        /* connect handler */
        let apps = appstate.clone();
        let notif = try_e!(Notif::new(&apps.prefs.borrow()));
        appstate.audio.connect_handler(
            Box::new(move |s, u| match (s, u) {
                (AudioSignal::CardDisconnected, _) => (),
                (AudioSignal::CardError, _) => (),
                (AudioSignal::ValuesChanged, AudioUser::TrayIcon) => {
                    try_w!(notif.show_volume_notif(&apps.audio))
                },
                _ => (),
                }
            )
        );

    }
}




