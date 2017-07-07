use app_state::*;
use gtk;
use prefs::*;
use std::cell::RefCell;
use std::rc::Rc;
use ui_popup_menu::*;
use ui_popup_window::*;
use ui_prefs_dialog::PrefsDialog;
use ui_tray_icon::*;



pub struct Gui {
    _cant_construct: (),
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
                   _cant_construct: (),
                   tray_icon: TrayIcon::new(prefs).unwrap(),
                   popup_window: PopupWindow::new(builder_popup_window),
                   popup_menu: PopupMenu::new(builder_popup_menu),
                   prefs_dialog: RefCell::new(None),
               };
    }
}


pub fn init(appstate: Rc<AppS>) {
    {
        let mut apps = appstate.clone();
        // appstate.audio.connect_handler(
        // Box::new(move |s, u| match (s, u) {
        // (AudioSignal::ValuesChanged, AudioUser::Unknown) => {
        // debug!("External volume change!");

        // }
        // _ => debug!("Nix"),
        // }),
        // );

    }

    init_tray_icon(appstate.clone());
    init_popup_window(appstate.clone());
    init_popup_menu(appstate.clone());
}
