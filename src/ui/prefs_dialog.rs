//! The preferences window subsystem, when the user clicks the "Preferences"
//! menu item on the popup menu.


use app_state::*;
use audio::frontend::*;
use errors::*;
use gdk;
use gtk::ResponseType;
use gtk::prelude::*;
use gtk;
use prefs::*;
use std::cell::RefCell;
use std::rc::Rc;
use support::audio::*;
use ui::hotkey_dialog::HotkeyDialog;



/// The main preferences dialog, holding all the relevant subwidgets we
/// need to convert its state to preferences and back.
pub struct PrefsDialog {
    _cant_construct: (),
    prefs_dialog: gtk::Dialog,
    notebook: gtk::Notebook,

    /* DevicePrefs */
    card_combo: gtk::ComboBoxText,
    chan_combo: gtk::ComboBoxText,

    /* ViewPrefs */
    vol_meter_draw_check: gtk::CheckButton,
    vol_meter_pos_spin: gtk::SpinButton,
    vol_meter_color_button: gtk::ColorButton,
    system_theme: gtk::CheckButton,

    /* BehaviorPrefs */
    unmute_on_vol_change: gtk::CheckButton,
    vol_control_entry: gtk::Entry,
    scroll_step_spin: gtk::SpinButton,
    fine_scroll_step_spin: gtk::SpinButton,
    middle_click_combo: gtk::ComboBoxText,
    custom_entry: gtk::Entry,

    /* NotifyPrefs */
    #[cfg(feature = "notify")]
    noti_enable_check: gtk::CheckButton,
    #[cfg(feature = "notify")]
    noti_timeout_spin: gtk::SpinButton,
    // pub noti_hotkey_check: gtk::CheckButton,
    #[cfg(feature = "notify")]
    noti_mouse_check: gtk::CheckButton,
    #[cfg(feature = "notify")]
    noti_popup_check: gtk::CheckButton,
    #[cfg(feature = "notify")]
    noti_ext_check: gtk::CheckButton,
    #[cfg(feature = "notify")]
    noti_hotkey_check: gtk::CheckButton,

    /* HotkeyPrefs */
    hotkeys_enable_check: gtk::CheckButton,
    hotkeys_mute_label: gtk::Label,
    hotkeys_up_label: gtk::Label,
    hotkeys_down_label: gtk::Label,

    /* Hotkey stuff (not prefs) */
    hotkeys_mute_eventbox: gtk::EventBox,
    hotkeys_up_eventbox: gtk::EventBox,
    hotkeys_down_eventbox: gtk::EventBox,

    hotkey_dialog: RefCell<Option<HotkeyDialog>>,
}

impl PrefsDialog {
    fn new() -> PrefsDialog {
        let builder = gtk::Builder::new_from_string(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/data/ui/prefs-dialog.glade"
        )));
        let prefs_dialog = PrefsDialog {
            _cant_construct: (),
            prefs_dialog: builder.get_object("prefs_dialog").unwrap(),
            notebook: builder.get_object("notebook").unwrap(),

            /* DevicePrefs */
            card_combo: builder.get_object("card_combo").unwrap(),
            chan_combo: builder.get_object("chan_combo").unwrap(),

            /* ViewPrefs */
            vol_meter_draw_check: builder
                .get_object("vol_meter_draw_check")
                .unwrap(),
            vol_meter_pos_spin: builder
                .get_object("vol_meter_pos_spin")
                .unwrap(),
            vol_meter_color_button: builder
                .get_object("vol_meter_color_button")
                .unwrap(),
            system_theme: builder.get_object("system_theme").unwrap(),

            /* BehaviorPrefs */
            unmute_on_vol_change: builder
                .get_object("unmute_on_vol_change")
                .unwrap(),
            vol_control_entry: builder.get_object("vol_control_entry").unwrap(),
            scroll_step_spin: builder.get_object("scroll_step_spin").unwrap(),
            fine_scroll_step_spin: builder
                .get_object("fine_scroll_step_spin")
                .unwrap(),
            middle_click_combo: builder
                .get_object("middle_click_combo")
                .unwrap(),
            custom_entry: builder.get_object("custom_entry").unwrap(),

            /* NotifyPrefs */
            #[cfg(feature = "notify")]
            #[cfg(feature = "notify")]
            #[cfg(feature = "notify")]
            #[cfg(feature = "notify")]
            noti_enable_check: builder.get_object("noti_enable_check").unwrap(),
            #[cfg(feature = "notify")]
            noti_timeout_spin: builder.get_object("noti_timeout_spin").unwrap(),
            // noti_hotkey_check: builder.get_object("noti_hotkey_check").unwrap(),
            #[cfg(feature = "notify")]
            #[cfg(feature = "notify")]
            #[cfg(feature = "notify")]
            #[cfg(feature = "notify")]
            noti_mouse_check: builder.get_object("noti_mouse_check").unwrap(),
            #[cfg(feature = "notify")]
            noti_popup_check: builder.get_object("noti_popup_check").unwrap(),
            #[cfg(feature = "notify")]
            noti_ext_check: builder.get_object("noti_ext_check").unwrap(),
            #[cfg(feature = "notify")]
            noti_hotkey_check: builder.get_object("noti_hotkey_check").unwrap(),

            /* HotkeyPrefs */
            hotkeys_enable_check: builder
                .get_object("hotkeys_enable_check")
                .unwrap(),
            hotkeys_mute_label: builder
                .get_object("hotkeys_mute_label")
                .unwrap(),
            hotkeys_up_label: builder.get_object("hotkeys_up_label").unwrap(),
            hotkeys_down_label: builder
                .get_object("hotkeys_down_label")
                .unwrap(),

            /* Hotkey stuff (not prefs) */
            hotkeys_mute_eventbox: builder
                .get_object("hotkeys_mute_eventbox")
                .unwrap(),
            hotkeys_up_eventbox: builder
                .get_object("hotkeys_up_eventbox")
                .unwrap(),
            hotkeys_down_eventbox: builder
                .get_object("hotkeys_down_eventbox")
                .unwrap(),
            hotkey_dialog: RefCell::new(None),
        };

        #[cfg(feature = "notify")]
        let notify_tab: gtk::Box =
            builder.get_object("noti_vbox_enabled").unwrap();
        #[cfg(not(feature = "notify"))]
        let notify_tab: gtk::Box =
            builder.get_object("noti_vbox_disabled").unwrap();

        prefs_dialog.notebook.append_page(
            &notify_tab,
            Some(&gtk::Label::new(Some("Notifications"))),
        );
        return prefs_dialog;
    }


    /// Import the given preferences into the preferences dialog state.
    fn from_prefs(&self, prefs: &Prefs) {
        /* DevicePrefs */
        /* filled on show signal with audio info */
        self.card_combo.remove_all();
        self.chan_combo.remove_all();

        /* ViewPrefs */
        self.vol_meter_draw_check.set_active(
            prefs.view_prefs.draw_vol_meter,
        );
        self.vol_meter_pos_spin.set_value(
            prefs.view_prefs.vol_meter_offset as
                f64,
        );

        let rgba = gdk::RGBA {
            red: prefs.view_prefs.vol_meter_color.red,
            green: prefs.view_prefs.vol_meter_color.green,
            blue: prefs.view_prefs.vol_meter_color.blue,
            alpha: 1.0,
        };
        self.vol_meter_color_button.set_rgba(&rgba);
        self.system_theme.set_active(prefs.view_prefs.system_theme);

        /* BehaviorPrefs */
        self.unmute_on_vol_change.set_active(
            prefs
                .behavior_prefs
                .unmute_on_vol_change,
        );
        self.vol_control_entry.set_text(
            prefs
                .behavior_prefs
                .vol_control_cmd
                .as_ref()
                .unwrap_or(&String::from(""))
                .as_str(),
        );
        self.scroll_step_spin.set_value(
            prefs.behavior_prefs.vol_scroll_step,
        );
        self.fine_scroll_step_spin.set_value(
            prefs
                .behavior_prefs
                .vol_fine_scroll_step,
        );

        // TODO: make sure these values always match, must be a better way
        //       also check to_prefs()
        self.middle_click_combo.append_text("Toggle Mute");
        self.middle_click_combo.append_text("Show Preferences");
        self.middle_click_combo.append_text("Volume Control");
        self.middle_click_combo.append_text(
            "Custom Command (set below)",
        );
        self.middle_click_combo.set_active(
            prefs
                .behavior_prefs
                .middle_click_action
                .into(),
        );
        self.custom_entry.set_text(
            prefs
                .behavior_prefs
                .custom_command
                .as_ref()
                .unwrap_or(&String::from(""))
                .as_str(),
        );

        /* NotifyPrefs */
        #[cfg(feature = "notify")]
        {
            self.noti_enable_check.set_active(
                prefs
                    .notify_prefs
                    .enable_notifications,
            );
            self.noti_timeout_spin.set_value(
                prefs.notify_prefs.notifcation_timeout as
                    f64,
            );
            self.noti_mouse_check.set_active(
                prefs.notify_prefs.notify_mouse_scroll,
            );
            self.noti_popup_check.set_active(
                prefs.notify_prefs.notify_popup,
            );
            self.noti_ext_check.set_active(
                prefs.notify_prefs.notify_external,
            );
            self.noti_hotkey_check.set_active(
                prefs.notify_prefs.notify_hotkeys,
            );
        }

        /* hotkey prefs */
        self.hotkeys_enable_check.set_active(
            prefs.hotkey_prefs.enable_hotkeys,
        );
        self.hotkeys_mute_label.set_text(
            prefs
                .hotkey_prefs
                .mute_unmute_key
                .clone()
                .unwrap_or(String::from("(None)"))
                .as_str(),
        );
        self.hotkeys_up_label.set_text(
            prefs
                .hotkey_prefs
                .vol_up_key
                .clone()
                .unwrap_or(String::from("(None)"))
                .as_str(),
        );
        self.hotkeys_down_label.set_text(
            prefs
                .hotkey_prefs
                .vol_down_key
                .clone()
                .unwrap_or(String::from("(None)"))
                .as_str(),
        );
    }


    /// Export the dialog state to the `Prefs` struct, which can be used
    /// to write them to the config file.
    fn to_prefs(&self) -> Prefs {
        let card = self.card_combo.get_active_text();
        let channel = self.chan_combo.get_active_text();

        if card.is_none() || channel.is_none() {
            return Prefs::default();
        }

        let device_prefs = DevicePrefs {
            card: self.card_combo.get_active_text().unwrap(),
            channel: self.chan_combo.get_active_text().unwrap(),
        };

        let vol_meter_color = VolColor {
            red: (self.vol_meter_color_button.get_rgba().red),
            green: (self.vol_meter_color_button.get_rgba().green),
            blue: (self.vol_meter_color_button.get_rgba().blue),
        };

        let view_prefs = ViewPrefs {
            draw_vol_meter: self.vol_meter_draw_check.get_active(),
            vol_meter_offset: self.vol_meter_pos_spin.get_value_as_int(),
            system_theme: self.system_theme.get_active(),
            vol_meter_color,
        };

        let vol_control_cmd = self.vol_control_entry.get_text().and_then(|x| {
            if x.is_empty() { None } else { Some(x) }
        });

        let custom_command =
            self.custom_entry.get_text().and_then(|x| if x.is_empty() {
                None
            } else {
                Some(x)
            });

        let behavior_prefs = BehaviorPrefs {
            unmute_on_vol_change: self.unmute_on_vol_change.get_active(),
            vol_control_cmd,
            vol_scroll_step: self.scroll_step_spin.get_value(),
            vol_fine_scroll_step: self.fine_scroll_step_spin.get_value(),
            middle_click_action: self.middle_click_combo.get_active().into(),
            custom_command,
        };

        #[cfg(feature = "notify")]
        let notify_prefs = NotifyPrefs {
            enable_notifications: self.noti_enable_check.get_active(),
            notifcation_timeout: self.noti_timeout_spin.get_value_as_int() as
                i64,
            notify_mouse_scroll: self.noti_mouse_check.get_active(),
            notify_popup: self.noti_popup_check.get_active(),
            notify_external: self.noti_ext_check.get_active(),
            notify_hotkeys: self.noti_hotkey_check.get_active(),
        };

        let hotkey_prefs =
            HotkeyPrefs {
                enable_hotkeys: self.hotkeys_enable_check.get_active(),
                mute_unmute_key: self.hotkeys_mute_label.get_text().and_then(
                    |s| {
                        if s == "(None)" { None } else { Some(s) }
                    },
                ),
                vol_up_key: self.hotkeys_up_label.get_text().and_then(
                    |s| if s ==
                        "(None)"
                    {
                        None
                    } else {
                        Some(s)
                    },
                ),
                vol_down_key: self.hotkeys_down_label.get_text().and_then(
                    |s| if s ==
                        "(None)"
                    {
                        None
                    } else {
                        Some(s)
                    },
                ),
            };

        return Prefs {
            device_prefs,
            view_prefs,
            behavior_prefs,
            #[cfg(feature = "notify")]
            notify_prefs,
            hotkey_prefs,
        };

    }
}


/// Show the preferences dialog. This is created and destroyed dynamically
/// and not persistent across the application lifetime.
pub fn show_prefs_dialog<T>(appstate: &Rc<AppS<T>>)
where
    T: AudioFrontend + 'static,
{
    if appstate.gui.prefs_dialog.borrow().is_some() {
        return;
    }

    *appstate.gui.prefs_dialog.borrow_mut() = Some(PrefsDialog::new());
    init_prefs_dialog(&appstate);
    {
        let m_pd = appstate.gui.prefs_dialog.borrow();
        let prefs_dialog = &m_pd.as_ref().unwrap();
        let prefs_dialog_w = &prefs_dialog.prefs_dialog;

        prefs_dialog.from_prefs(&appstate.prefs.borrow());

        prefs_dialog_w.set_transient_for(&appstate.gui.popup_menu.menu_window);
        prefs_dialog_w.present();
    }
}


/// Initialize the internal prefs dialog handler that connects to the audio
/// system.
pub fn init_prefs_callback<T>(appstate: Rc<AppS<T>>)
where
    T: AudioFrontend + 'static,
{
    let apps = appstate.clone();
    appstate.audio.connect_handler(Box::new(move |s, u| {
        /* skip if prefs window is not present */
        if apps.gui.prefs_dialog.borrow().is_none() {
            return;
        }

        match (s, u) {
            (AudioSignal::CardInitialized, _) => (),
            (AudioSignal::CardCleanedUp, _) => {
                fill_card_combo(&apps);
                fill_chan_combo(&apps, None);
            }
            _ => (),
        }
    }));
}


/// Initialize the preferences dialog gtk callbacks.
fn init_prefs_dialog<T>(appstate: &Rc<AppS<T>>)
where
    T: AudioFrontend + 'static,
{

    /* prefs_dialog.connect_show */
    {
        let apps = appstate.clone();
        let m_pd = appstate.gui.prefs_dialog.borrow();
        let pd = m_pd.as_ref().unwrap();
        pd.prefs_dialog.connect_show(move |_| {
            fill_card_combo(&apps);
            fill_chan_combo(&apps, None);
        });
    }

    /* card_combo.connect_changed */
    {
        let apps = appstate.clone();
        let m_cc = appstate.gui.prefs_dialog.borrow();
        let card_combo = &m_cc.as_ref().unwrap().card_combo;

        card_combo.connect_changed(move |_| {
            let m_cc = apps.gui.prefs_dialog.borrow();
            let card_combo = &m_cc.as_ref().unwrap().card_combo;
            let card_name = card_combo.get_active_text().unwrap();
            fill_chan_combo(&apps, Some(card_name));
            return;
        });
    }

    /* prefs_dialog.connect_response */
    {
        let apps = appstate.clone();
        let m_pd = appstate.gui.prefs_dialog.borrow();
        let pd = m_pd.as_ref().unwrap();
        pd.prefs_dialog.connect_response(move |_, response_id| {

            if response_id == ResponseType::Ok.into() ||
                response_id == ResponseType::Apply.into()
            {
                let mut prefs = apps.prefs.borrow_mut();
                let prefs_dialog = apps.gui.prefs_dialog.borrow();
                *prefs = prefs_dialog.as_ref().unwrap().to_prefs();

            }

            if response_id != ResponseType::Apply.into() {
                let mut prefs_dialog = apps.gui.prefs_dialog.borrow_mut();
                prefs_dialog.as_ref().unwrap().prefs_dialog.destroy();
                *prefs_dialog = None;
            }

            if response_id == ResponseType::Ok.into() ||
                response_id == ResponseType::Apply.into()
            {
                try_w!(apps.update_popup_window());
                try_w!(apps.update_tray_icon());
                let _ = result_warn!(
                    apps.update_hotkeys(),
                    Some(&apps.gui.popup_menu.menu_window)
                );
                apps.update_notify();
                try_w!(apps.update_audio(AudioUser::PrefsWindow));
                let _ = result_warn!(
                    apps.update_config(),
                    Some(&apps.gui.popup_menu.menu_window)
                );
            }

        });
    }

    /* prefs_dialog.hotkeys_mute_eventbox */
    {
        let apps = appstate.clone();
        let m_pd = appstate.gui.prefs_dialog.borrow();
        let pd = m_pd.as_ref().unwrap();

        pd.hotkeys_mute_eventbox.connect_button_press_event(
            move |w, e| {
                return Inhibit(
                    on_hotkey_event_box_button_press_event(&apps, &w, e),
                );
            },
        );
    }

    /* prefs_dialog.hotkeys_up_eventbox */
    {
        let apps = appstate.clone();
        let m_pd = appstate.gui.prefs_dialog.borrow();
        let pd = m_pd.as_ref().unwrap();

        pd.hotkeys_up_eventbox.connect_button_press_event(
            move |w, e| {
                return Inhibit(
                    on_hotkey_event_box_button_press_event(&apps, &w, e),
                );
            },
        );
    }

    /* prefs_dialog.hotkeys_down_eventbox */
    {
        let apps = appstate.clone();
        let m_pd = appstate.gui.prefs_dialog.borrow();
        let pd = m_pd.as_ref().unwrap();

        pd.hotkeys_down_eventbox.connect_button_press_event(
            move |w, e| {
                return Inhibit(
                    on_hotkey_event_box_button_press_event(&apps, &w, e),
                );
            },
        );
    }
}


/// Fill the card combo box in the Devices tab.
fn fill_card_combo<T>(appstate: &AppS<T>)
where
    T: AudioFrontend,
{
    let m_cc = appstate.gui.prefs_dialog.borrow();
    let card_combo = &m_cc.as_ref().unwrap().card_combo;
    card_combo.remove_all();
    let audio = &appstate.audio;

    /* set card combo */
    let cur_card_name =
        try_w!(audio.card_name(), "Can't get current card name!");
    let available_card_names = get_playable_card_names();

    /* set_active_id doesn't work, so save the index */
    let mut c_index: i32 = -1;
    for i in 0..available_card_names.len() {
        let name = available_card_names.get(i).unwrap();
        if *name == cur_card_name {
            c_index = i as i32;
        }
        card_combo.append_text(&name);
    }

    // TODO, block signal?
    card_combo.set_active(c_index);
}


/// Fill the channel combo box in the Devices tab.
fn fill_chan_combo<T>(appstate: &AppS<T>, cardname: Option<String>)
where
    T: AudioFrontend,
{
    let m_cc = appstate.gui.prefs_dialog.borrow();
    let chan_combo = &m_cc.as_ref().unwrap().chan_combo;
    chan_combo.remove_all();

    let audio = &appstate.audio;
    let available_chan_names = match cardname {
        Some(name) => get_playable_chan_names(name),
        None => audio.playable_chan_names(),
    };

    /* set chan combo */
    let cur_chan_name = try_w!(audio.chan_name());

    /* set_active_id doesn't work, so save the index */
    let mut c_index: i32 = -1;
    for i in 0..available_chan_names.len() {
        let name = available_chan_names.get(i).unwrap();
        if *name == cur_chan_name {
            c_index = i as i32;
        }
        chan_combo.append_text(&name);
    }

    /* TODO, block signal?`*/
    chan_combo.set_active(c_index);

}


fn on_hotkey_event_box_button_press_event<T>(
    appstate: &AppS<T>,
    widget: &gtk::EventBox,
    event: &gdk::EventButton,
) -> bool
where
    T: AudioFrontend,
{
    let borrow = appstate.gui.prefs_dialog.borrow();
    let prefs_dialog = &borrow.as_ref().unwrap();
    /* we want a left-click */
    if event.get_button() != 1 {
        return false;
    }

    /* we want it to be double-click */
    if event.get_event_type() != gdk::EventType::DoubleButtonPress {
        return false;
    }

    let (hotkey_label, hotkey) = {
        if *widget == prefs_dialog.hotkeys_mute_eventbox {
            (
                prefs_dialog.hotkeys_mute_label.clone(),
                String::from("Mute/Unmute"),
            )
        } else if *widget == prefs_dialog.hotkeys_up_eventbox {
            (
                prefs_dialog.hotkeys_up_label.clone(),
                String::from("Volume Up"),
            )
        } else if *widget == prefs_dialog.hotkeys_down_eventbox {
            (
                prefs_dialog.hotkeys_down_label.clone(),
                String::from("Volume Down"),
            )
        } else {
            warn!("Unknown hotkey eventbox");
            return false;
        }
    };

    /* Ensure there's no dialog already running */
    if prefs_dialog.hotkey_dialog.borrow().is_some() {
        return false;
    }

    /* Unbind hotkeys */
    appstate.hotkeys.borrow().unbind();

    /* Run the hotkey dialog */
    let hotkey_dialog = &prefs_dialog.hotkey_dialog;
    *hotkey_dialog.borrow_mut() =
        Some(HotkeyDialog::new(&prefs_dialog.prefs_dialog, hotkey));
    let key_pressed = hotkey_dialog.borrow().as_ref().unwrap().run();
    *hotkey_dialog.borrow_mut() = None;

    /* Bind hotkeys */
    appstate.hotkeys.borrow().bind();

    /* Check the response */
    match key_pressed {
        Ok(k) => {
            println!("k: {}", k);
            if k.eq_ignore_ascii_case("<Primary>c") {
                hotkey_label.set_text("(None)");
            } else {
                hotkey_label.set_text(k.as_str());
            }
        }
        Err(Error(ErrorKind::GtkResponseCancel(msg), _)) => {
            info!("{}", ErrorKind::GtkResponseCancel(msg));
            return false;
        }
        Err(e) => {
            // Could not grab hotkey, most likely
            error_dialog!(
                e.description(),
                Some(&appstate.gui.popup_menu.menu_window)
            );
            warn!("{}", e);
            return false;
        }
    }

    return false;
}
