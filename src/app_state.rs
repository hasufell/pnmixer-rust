use audio::Audio;
use gdk;
use gtk;
use gtk::ComboBoxTextExt;
use gtk::ComboBoxExt;
use gtk::ToggleButtonExt;
use gtk::SpinButtonExt;
use gtk::ColorChooserExt;
use gtk::EntryExt;
use prefs::*;
use ui_tray_icon::TrayIcon;
use std::cell::RefCell;



// TODO: destructors

// TODO: glade stuff, config, alsacard
pub struct AppS {
    pub gui: Gui,
    pub audio: Audio,
    pub prefs: RefCell<Prefs>,
}


impl AppS {
    pub fn new() -> AppS {
        let builder_popup_window =
            gtk::Builder::new_from_string(include_str!("../data/ui/popup-window.glade"));
        let builder_popup_menu = gtk::Builder::new_from_string(include_str!("../data/ui/popup-menu.glade"));
        let prefs = RefCell::new(Prefs::new().unwrap());
        let gui =
            Gui::new(builder_popup_window, builder_popup_menu, &prefs.borrow());

        return AppS {
                   gui: gui,
                   audio: Audio::new(None, Some(String::from("Master")))
                       .unwrap(),
                   prefs: prefs,
               };
    }
}


pub struct Gui {
    pub tray_icon: TrayIcon,
    pub popup_window: PopupWindow,
    pub popup_menu: PopupMenu,
    /* prefs_dialog is dynamically created and destroyed */
    pub prefs_dialog: RefCell<Option<PrefsDialog>>,
}


impl Gui {
    pub fn new(builder_popup_window: gtk::Builder,
               builder_popup_menu: gtk::Builder,
               prefs: &Prefs)
               -> Gui {
        return Gui {
                   tray_icon: TrayIcon::new(prefs).unwrap(),
                   popup_window: PopupWindow::new(builder_popup_window),
                   popup_menu: PopupMenu::new(builder_popup_menu),
                   prefs_dialog: RefCell::new(None),
               };
    }
}


create_builder_item!(PopupWindow,
                     popup_window: gtk::Window,
                     vol_scale_adj: gtk::Adjustment,
                     vol_scale: gtk::Scale,
                     mute_check: gtk::CheckButton);


create_builder_item!(PopupMenu,
                     menu_window: gtk::Window,
                     menubar: gtk::MenuBar,
                     menu: gtk::Menu,
                     about_item: gtk::MenuItem,
                     prefs_item: gtk::MenuItem);


pub struct PrefsDialog {
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
