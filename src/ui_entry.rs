use app_state::*;
use audio::{AudioUser, AudioSignal};
use gtk::DialogExt;
use gtk::MessageDialogExt;
use gtk::WidgetExt;
use gtk::WindowExt;
use gtk;
use gtk_sys::{GTK_DIALOG_DESTROY_WITH_PARENT, GTK_RESPONSE_YES};
use notif::*;
use prefs::*;
use std::cell::RefCell;
use std::rc::Rc;
use support_audio::*;
use ui_popup_menu::*;
use ui_popup_window::*;
use ui_prefs_dialog::*;
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
        /* "global" audio signal handler */
        let apps = appstate.clone();
        appstate.audio.connect_handler(
        Box::new(move |s, u| match (s, u) {
            (AudioSignal::CardDisconnected, _) => {
                 try_w!(audio_reload(&apps.audio,
                         &apps.prefs.borrow(),
                         AudioUser::Unknown));
            },
            (AudioSignal::CardError, _) => {
                if run_audio_error_dialog(&apps.gui.popup_menu.menu_window) == (GTK_RESPONSE_YES as i32) {
                     try_w!(audio_reload(&apps.audio,
                             &apps.prefs.borrow(),
                             AudioUser::Unknown));
                }
            },
            _ => (),
            }
        ));

    }

    init_tray_icon(appstate.clone());
    init_popup_window(appstate.clone());
    init_popup_menu(appstate.clone());
    init_prefs_callback(appstate.clone());
}


fn run_audio_error_dialog(parent: &gtk::Window) -> i32 {
    error!("Connection with audio failed, you probably need to restart pnmixer.");

    let dialog = gtk::MessageDialog::new(
            Some(parent),
            gtk::DIALOG_DESTROY_WITH_PARENT,
            gtk::MessageType::Error,
            gtk::ButtonsType::YesNo,
            "Warning: Connection to sound system failed."
        );
    dialog.set_property_secondary_text(Some("Do you want to re-initialize the audio connection ?

If you do not, you will either need to restart PNMixer
or select the 'Reload Audio' option in the right-click
menu in order for PNMixer to function."));

    dialog.set_title("PNMixer-rs Error");

    let resp = dialog.run();
    dialog.destroy();

    return resp;
}
