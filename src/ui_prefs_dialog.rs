use app_state::*;
use audio::{AudioUser, AudioSignal};
use errors::*;
use gdk;
use gtk::ResponseType;
use gtk::prelude::*;
use gtk;
use prefs::*;
use std::rc::Rc;
use support_alsa::*;
use support_audio::*;



// TODO: reference count leak



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
}

impl PrefsDialog {
    fn new() -> PrefsDialog {
        let builder =
            gtk::Builder::new_from_string(include_str!(concat!(env!("CARGO_MANIFEST_DIR"),
                                                               "/data/ui/prefs-dialog.glade")));
        let prefs_dialog = PrefsDialog {
            _cant_construct: (),
            prefs_dialog: builder.get_object("prefs_dialog").unwrap(),
            notebook: builder.get_object("notebook").unwrap(),

            card_combo: builder.get_object("card_combo").unwrap(),
            chan_combo: builder.get_object("chan_combo").unwrap(),

            vol_meter_draw_check: builder.get_object("vol_meter_draw_check")
                .unwrap(),
            vol_meter_pos_spin: builder.get_object("vol_meter_pos_spin")
                .unwrap(),
            vol_meter_color_button: builder.get_object("vol_meter_color_button")
                .unwrap(),
            system_theme: builder.get_object("system_theme").unwrap(),

            vol_control_entry: builder.get_object("vol_control_entry").unwrap(),
            scroll_step_spin: builder.get_object("scroll_step_spin").unwrap(),
            fine_scroll_step_spin: builder.get_object("fine_scroll_step_spin")
                .unwrap(),
            middle_click_combo: builder.get_object("middle_click_combo")
                .unwrap(),
            custom_entry: builder.get_object("custom_entry").unwrap(),

            #[cfg(feature = "notify")]
            noti_enable_check: builder.get_object("noti_enable_check").unwrap(),
            #[cfg(feature = "notify")]
            noti_timeout_spin: builder.get_object("noti_timeout_spin").unwrap(),
            // noti_hotkey_check: builder.get_object("noti_hotkey_check").unwrap(),
            #[cfg(feature = "notify")]
            noti_mouse_check: builder.get_object("noti_mouse_check").unwrap(),
            #[cfg(feature = "notify")]
            noti_popup_check: builder.get_object("noti_popup_check").unwrap(),
            #[cfg(feature = "notify")]
            noti_ext_check: builder.get_object("noti_ext_check").unwrap(),
        };

        #[cfg(feature = "notify")]
        let notify_tab: gtk::Box = builder.get_object("noti_vbox_enabled").unwrap();
        #[cfg(not(feature = "notify"))]
        let notify_tab: gtk::Box = builder.get_object("noti_vbox_disabled").unwrap();

        prefs_dialog.notebook.append_page(&notify_tab,
                                          Some(&gtk::Label::new(Some("Notifications"))));
        return prefs_dialog;
    }


    fn from_prefs(&self, prefs: &Prefs) {
        /* DevicePrefs */
        /* filled on show signal with audio info */
        self.card_combo.remove_all();
        self.chan_combo.remove_all();

        /* ViewPrefs */
        self.vol_meter_draw_check.set_active(prefs.view_prefs.draw_vol_meter);
        self.vol_meter_pos_spin.set_value(prefs.view_prefs.vol_meter_offset as
                                          f64);

        // TODO don't convert like that
        let rgba = gdk::RGBA {
            red: prefs.view_prefs.vol_meter_color.red as f64 / 255.0,
            green: prefs.view_prefs.vol_meter_color.green as f64 / 255.0,
            blue: prefs.view_prefs.vol_meter_color.blue as f64 / 255.0,
            alpha: 1.0,
        };
        self.vol_meter_color_button.set_rgba(&rgba);
        self.system_theme.set_active(prefs.view_prefs.system_theme);

        /* BehaviorPrefs */
        self.vol_control_entry.set_text(prefs.behavior_prefs
                                            .vol_control_cmd
                                            .as_ref()
                                            .unwrap_or(&String::from(""))
                                            .as_str());
        self.scroll_step_spin.set_value(prefs.behavior_prefs.vol_scroll_step);
        self.fine_scroll_step_spin
            .set_value(prefs.behavior_prefs.vol_fine_scroll_step);

        // TODO: make sure these values always match, must be a better way
        //       also check to_prefs()
        self.middle_click_combo.append_text("Toggle Mute");
        self.middle_click_combo.append_text("Show Preferences");
        self.middle_click_combo.append_text("Volume Control");
        self.middle_click_combo.append_text("Custom Command (set below)");
        self.middle_click_combo.set_active(prefs.behavior_prefs
                                               .middle_click_action
                                               .into());
        self.custom_entry.set_text(prefs.behavior_prefs
                                       .custom_command
                                       .as_ref()
                                       .unwrap_or(&String::from(""))
                                       .as_str());

        /* NotifyPrefs */
        #[cfg(feature = "notify")]
        self.noti_enable_check
            .set_active(prefs.notify_prefs.enable_notifications);
        #[cfg(feature = "notify")]
        self.noti_timeout_spin
            .set_value(prefs.notify_prefs.notifcation_timeout as f64);
        #[cfg(feature = "notify")]
        self.noti_mouse_check
            .set_active(prefs.notify_prefs.notify_mouse_scroll);
        #[cfg(feature = "notify")]
        self.noti_popup_check.set_active(prefs.notify_prefs.notify_popup);
        #[cfg(feature = "notify")]
        self.noti_ext_check.set_active(prefs.notify_prefs.notify_external);
    }


    fn to_prefs(&self) -> Prefs {
        // TODO: remove duplication with default instance
        let device_prefs =
            DevicePrefs {
                card: self.card_combo
                    .get_active_text()
                    .unwrap_or(String::from("(default)")),
                channel: self.chan_combo
                    .get_active_text()
                    .unwrap_or(String::from("Master")),
            };

        // TODO don't convert like that
        let vol_meter_color = VolColor {
            red: (self.vol_meter_color_button.get_rgba().red * 255.0) as u8,
            green: (self.vol_meter_color_button.get_rgba().green * 255.0) as u8,
            blue: (self.vol_meter_color_button.get_rgba().blue * 255.0) as u8,
        };

        let view_prefs = ViewPrefs {
            draw_vol_meter: self.vol_meter_draw_check.get_active(),
            vol_meter_offset: self.vol_meter_pos_spin.get_value_as_int(),
            system_theme: self.system_theme.get_active(),
            vol_meter_color,
        };

        let vol_control_cmd =
            self.vol_control_entry.get_text().and_then(|x| if x.is_empty() {
                                                           None
                                                       } else {
                                                           Some(x)
                                                       });

        let custom_command =
            self.custom_entry.get_text().and_then(|x| if x.is_empty() {
                                                      None
                                                  } else {
                                                      Some(x)
                                                  });

        let behavior_prefs = BehaviorPrefs {
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
        };

        return Prefs {
                   device_prefs,
                   view_prefs,
                   behavior_prefs,
                   #[cfg(feature = "notify")]
                   notify_prefs,
               };

    }
}


pub fn show_prefs_dialog(appstate: &Rc<AppS>) {
    if appstate.gui
           .prefs_dialog
           .borrow()
           .is_some() {
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


pub fn init_prefs_callback(appstate: Rc<AppS>) {
    let apps = appstate.clone();
    appstate.audio.connect_handler(Box::new(move |s, u| {
        /* skip if prefs window is not present */
        if apps.gui
               .prefs_dialog
               .borrow()
               .is_none() {
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


fn init_prefs_dialog(appstate: &Rc<AppS>) {

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
               response_id == ResponseType::Apply.into() {
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
               response_id == ResponseType::Apply.into() {
                // TODO: update hotkeys
                try_w!(apps.update_notify());
                try_w!(apps.update_tray_icon());
                try_w!(apps.update_popup_window());
                try_w!(apps.update_audio(AudioUser::PrefsWindow));
                try_w!(apps.update_config());
               }

        });
    }
}


fn fill_card_combo(appstate: &AppS) {
    let m_cc = appstate.gui.prefs_dialog.borrow();
    let card_combo = &m_cc.as_ref().unwrap().card_combo;
    card_combo.remove_all();
    let acard = appstate.audio.acard.borrow();

    /* set card combo */
    let cur_card_name = try_w!(acard.card_name(),
                               "Can't get current card name!");
    let available_card_names = get_playable_alsa_card_names();

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


fn fill_chan_combo(appstate: &AppS, cardname: Option<String>) {
    let m_cc = appstate.gui.prefs_dialog.borrow();
    let chan_combo = &m_cc.as_ref().unwrap().chan_combo;
    chan_combo.remove_all();

    let cur_acard = appstate.audio.acard.borrow();
    let card = match cardname {
        Some(name) => try_w!(get_alsa_card_by_name(name).from_err()),
        None => cur_acard.as_ref().card,
    };

    /* set chan combo */
    let cur_chan_name = try_w!(cur_acard.chan_name());
    let mixer = try_w!(get_mixer(&card));
    let available_chan_names = get_playable_selem_names(&mixer);

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
