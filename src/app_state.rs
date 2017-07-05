use audio::Audio;
use gtk;
use prefs::Prefs;
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
