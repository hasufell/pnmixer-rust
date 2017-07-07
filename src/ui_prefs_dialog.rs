use app_state::*;
use audio::AudioUser;
use gdk;
use gtk::ResponseType;
use gtk::prelude::*;
use gtk;
use prefs::*;
use std::rc::Rc;
use support_alsa::*;



// TODO: misbehavior when popup_window is open



pub struct PrefsDialog {
    _cant_construct: (),
    pub prefs_dialog: gtk::Dialog,

    /* DevicePrefs */
    pub card_combo: gtk::ComboBoxText,
    pub chan_combo: gtk::ComboBoxText,

    /* ViewPrefs */
    pub vol_meter_draw_check: gtk::CheckButton,
    pub vol_meter_pos_spin: gtk::SpinButton,
    pub vol_meter_color_button: gtk::ColorButton,
    pub system_theme: gtk::CheckButton,

    /* BehaviorPrefs */
    pub vol_control_entry: gtk::Entry,
    pub scroll_step_spin: gtk::SpinButton,
    pub middle_click_combo: gtk::ComboBoxText,
    pub custom_entry: gtk::Entry,

    /* NotifyPrefs */
    pub noti_enable_check: gtk::CheckButton,
    pub noti_timeout_spin: gtk::SpinButton,
    // pub noti_hotkey_check: gtk::CheckButton,
    pub noti_mouse_check: gtk::CheckButton,
    pub noti_popup_check: gtk::CheckButton,
    pub noti_ext_check: gtk::CheckButton,
}

impl PrefsDialog {
    pub fn new() -> PrefsDialog {
        let builder = gtk::Builder::new_from_string(include_str!("../data/ui/prefs-dialog.glade"));
        let prefs_dialog = PrefsDialog {
            _cant_construct: (),
            prefs_dialog: builder.get_object("prefs_dialog").unwrap(),

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
            middle_click_combo: builder.get_object("middle_click_combo")
                .unwrap(),
            custom_entry: builder.get_object("custom_entry").unwrap(),

            noti_enable_check: builder.get_object("noti_enable_check").unwrap(),
            noti_timeout_spin: builder.get_object("noti_timeout_spin").unwrap(),
            // noti_hotkey_check: builder.get_object("noti_hotkey_check").unwrap(),
            noti_mouse_check: builder.get_object("noti_mouse_check").unwrap(),
            noti_popup_check: builder.get_object("noti_popup_check").unwrap(),
            noti_ext_check: builder.get_object("noti_ext_check").unwrap(),
        };


        return prefs_dialog;
    }


    pub fn from_prefs(&self, prefs: &Prefs) {
        /* DevicePrefs */
        self.card_combo.remove_all();
        self.card_combo.append_text(prefs.device_prefs.card.as_str());
        self.card_combo.set_active(0);

        self.chan_combo.remove_all();
        self.chan_combo.append_text(prefs.device_prefs.channel.as_str());
        self.chan_combo.set_active(0);

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

        // TODO: make sure these values always match, must be a better way
        //       also check to_prefs()
        self.middle_click_combo.append_text("Toggle Mute");
        self.middle_click_combo.append_text("Show Preferences");
        self.middle_click_combo.append_text("Volume Control");
        self.middle_click_combo.append_text("Custom Command");
        self.middle_click_combo.set_active(prefs.behavior_prefs
                                               .middle_click_action
                                               .into());
        self.custom_entry.set_text(prefs.behavior_prefs
                                       .custom_command
                                       .as_ref()
                                       .unwrap_or(&String::from(""))
                                       .as_str());

        /* NotifyPrefs */
        self.noti_enable_check
            .set_active(prefs.notify_prefs.enable_notifications);
        self.noti_timeout_spin
            .set_value(prefs.notify_prefs.notifcation_timeout as f64);
        self.noti_mouse_check
            .set_active(prefs.notify_prefs.notify_mouse_scroll);
        self.noti_popup_check.set_active(prefs.notify_prefs.notify_popup);
        self.noti_ext_check.set_active(prefs.notify_prefs.notify_external);
    }


    pub fn to_prefs(&self) -> Prefs {
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
            middle_click_action: self.middle_click_combo.get_active().into(),
            custom_command,
        };

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
                   notify_prefs,
               };

    }
}


pub fn show_prefs_dialog(appstate: Rc<AppS>) {
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
        // TODO: destruct PrefsDialog when clicking Ok/Apply
        prefs_dialog_w.present();
    }
}


/* TODO: do the references get dropped when the dialog window is gone? */
pub fn init_prefs_dialog(appstate: &Rc<AppS>) {

    /* prefs_dialog.connect_show */
    {
        let apps = appstate.clone();
        let m_pd = appstate.gui.prefs_dialog.borrow();
        let pd = m_pd.as_ref().unwrap();
        pd.prefs_dialog.connect_show(move |_| { on_prefs_dialog_show(&apps); });
    }

    /* prefs_dialog.connect_show */
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
                // TODO: update popup, tray_icon, hotkeys, notification and audio
                try_w!(apps.update_tray_icon());
                try_w!(apps.update_popup_window());
                let prefs = apps.prefs.borrow_mut();
                try_w!(prefs.store_config());
               }

        });
    }

    // TODO: fix combo box behavior and filling
    /*  DEVICE TAB */

    /* card_combo.connect_changed */
    {
        let apps = appstate.clone();
        let m_cc = appstate.gui.prefs_dialog.borrow();
        let card_combo = &m_cc.as_ref().unwrap().card_combo;

        // TODO: refill channel combo
        card_combo.connect_changed(move |_| { on_card_combo_changed(&apps); });
    }
    /* card_combo.connect_changed */
    {
        let apps = appstate.clone();
        let m_cc = appstate.gui.prefs_dialog.borrow();
        let chan_combo = &m_cc.as_ref().unwrap().chan_combo;

        chan_combo.connect_changed(move |_| { on_chan_combo_changed(&apps); });
    }
}


fn on_prefs_dialog_show(appstate: &AppS) {
    let m_cc = appstate.gui.prefs_dialog.borrow();
    let card_combo = &m_cc.as_ref().unwrap().card_combo;
    let m_cc = appstate.gui.prefs_dialog.borrow();
    let chan_combo = &m_cc.as_ref().unwrap().chan_combo;
    let acard = appstate.audio.acard.borrow();


    /* set card combo */
    let cur_card_name = try_w!(acard.card_name(),
                               "Can't get current card name!");
    let available_card_names = get_alsa_card_names();

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



    /* set chan combo */
    let cur_chan_name = try_w!(acard.chan_name());
    let available_chan_names = get_selem_names(&acard.mixer);

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


fn on_card_combo_changed(appstate: &AppS) {
    let m_cc = appstate.gui.prefs_dialog.borrow();
    let card_combo = &m_cc.as_ref().unwrap().card_combo;
    let m_cc = appstate.gui.prefs_dialog.borrow();
    let chan_combo = &m_cc.as_ref().unwrap().chan_combo;
    let active_card_item =
        try_w!(card_combo.get_active_text().ok_or("No active Card item found"));
    let active_chan_item = chan_combo.get_active_id();
    let cur_card_name = {
        let acard = appstate.audio.acard.borrow();
        try_w!(acard.card_name(), "Can't get current card name!")
    };

    if active_card_item != cur_card_name {
        appstate.audio.switch_acard(Some(cur_card_name),
                                    active_chan_item,
                                    AudioUser::PrefsWindow);
    }
}


fn on_chan_combo_changed(appstate: &AppS) {
    let m_cc = appstate.gui.prefs_dialog.borrow();
    let card_combo = &m_cc.as_ref().unwrap().card_combo;
    let m_cc = appstate.gui.prefs_dialog.borrow();
    let chan_combo = &m_cc.as_ref().unwrap().chan_combo;
    let active_chan_item =
        try_w!(chan_combo.get_active_text().ok_or("No active Chan item found"));
    let cur_card_name = {
        let acard = appstate.audio.acard.borrow();
        acard.card_name().ok()
    };
    let cur_chan_name = {
        let acard = appstate.audio.acard.borrow();
        try_w!(acard.chan_name())
    };

    if active_chan_item != cur_chan_name {
        appstate.audio.switch_acard(cur_card_name,
                                    Some(active_chan_item),
                                    AudioUser::PrefsWindow);
    }
}


