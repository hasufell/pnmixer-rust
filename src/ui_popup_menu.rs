use app_state::*;
use gtk::prelude::*;
use std::rc::Rc;
use gtk;



const VERSION: &'static str = env!("CARGO_PKG_VERSION");


pub fn init_popup_menu(appstate: Rc<AppS>) {

    /* about_item.connect_activate_link */
    {
        let apps = appstate.clone();
        let about_item = &appstate.clone()
                              .gui
                              .popup_menu
                              .about_item;
        about_item.connect_activate(move |_| {
                                        on_about_item_activate(&apps);
                                    });
    }

    /* about_item.connect_activate_link */
    {
        let apps = appstate.clone();
        let prefs_item = &appstate.clone()
                              .gui
                              .popup_menu
                              .prefs_item;
        prefs_item.connect_activate(move |_| {
                                        on_prefs_item_activate(&apps);
                                    });
    }
}


fn on_about_item_activate(appstate: &AppS) {
    let popup_menu = &appstate.gui.popup_menu.menu_window;
    let about_dialog = create_about_dialog();
    about_dialog.set_skip_taskbar_hint(true);
    about_dialog.set_transient_for(popup_menu);
    about_dialog.run();
    about_dialog.destroy();
}


fn create_about_dialog() -> gtk::AboutDialog {
    let about_dialog: gtk::AboutDialog = gtk::AboutDialog::new();

    about_dialog.set_license(Some("PNMixer is free software; you can redistribute it and/or modify it
under the terms of the GNU General Public License v3 as published
by the Free Software Foundation.

PNMixer is distributed in the hope that it will be useful, but
WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.
See the GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with PNMixer; if not, write to the Free Software Foundation,
Inc., 51 Franklin Street, Fifth Floor, Boston, MA 02110-1301, USA."));
    about_dialog.set_copyright(Some("Copyright © 2017 Julian Ospald"));
    about_dialog.set_authors(&["Julian Ospald"]);
    about_dialog.set_artists(&["Paul Davey"]);
    about_dialog.set_program_name("pnmixer-rs");
    about_dialog.set_logo_icon_name("pnmixer");
    about_dialog.set_version(VERSION);
    about_dialog.set_website("https://github.com/hasufell/pnmixer-rust");
    about_dialog.set_comments("A mixer for the system tray");

    return about_dialog;
}


fn on_prefs_item_activate(appstate: &AppS) {
    /* TODO: only create if needed */
    let prefs_dialog = &appstate.gui.prefs_dialog.prefs_dialog;
    let popup_menu = &appstate.gui.popup_menu.menu_window;

    prefs_dialog.set_transient_for(popup_menu);
    prefs_dialog.run();
    // prefs_dialog.destroy();
}
