#![allow(missing_docs)] // glade_helpers

//! The popup menu subsystem when the user right-clicks on the tray icon.
//!
//! Shows the menu with the following entries:
//!
//! * Mute
//! * Volume Control
//! * Preferences
//! * Reload Sound
//! * About
//! * Quit

use app_state::*;
use audio::AudioUser;
use gtk::prelude::*;
use gtk;
use std::rc::Rc;
use support_audio::*;
use support_cmd::*;
use ui_prefs_dialog::*;


const VERSION: &'static str = env!("CARGO_PKG_VERSION");



create_builder_item!(PopupMenu,
                     menu_window: gtk::Window,
                     menubar: gtk::MenuBar,
                     menu: gtk::Menu,
                     about_item: gtk::MenuItem,
                     mixer_item: gtk::MenuItem,
                     mute_item: gtk::MenuItem,
                     mute_check: gtk::CheckButton,
                     prefs_item: gtk::MenuItem,
                     quit_item: gtk::MenuItem,
                     reload_item: gtk::MenuItem);



/// Initialize the popup menu subsystem, registering all callbacks.
pub fn init_popup_menu(appstate: Rc<AppS>) {
    /* audio.connect_handler */
    {
        let apps = appstate.clone();
        appstate.audio.connect_handler(Box::new(move |s, u| {
            /* skip if window is hidden */
            if !apps.gui
                    .popup_menu
                    .menu
                    .get_visible() {
                return;
            }
            match (s, u) {
                (_, _) => set_mute_check(&apps),
            }
        }));

    }

    /* popup_menu.menu.connect_show */
    {
        let apps = appstate.clone();
        appstate.gui
            .popup_menu
            .menu
            .connect_show(move |_| set_mute_check(&apps));

    }

    /* mixer_item.connect_activate_link */
    {
        let apps = appstate.clone();
        let mixer_item = &appstate.gui.popup_menu.mixer_item;
        mixer_item.connect_activate(move |_| {
            let _ = result_warn!(execute_vol_control_command(&apps.prefs.borrow()),
                Some(&apps.gui.popup_menu.menu_window));
        });
    }

    /* mute_item.connect_activate_link */
    {
        let apps = appstate.clone();
        let mute_item = &appstate.gui.popup_menu.mute_item;
        mute_item.connect_activate(move |_| {
            if apps.audio.has_mute() {
                try_w!(apps.audio.toggle_mute(AudioUser::Popup));
            }
        });
    }

    /* about_item.connect_activate_link */
    {
        let apps = appstate.clone();
        let about_item = &appstate.gui.popup_menu.about_item;
        about_item.connect_activate(move |_| {
                                        on_about_item_activate(&apps);
                                    });
    }

    /* prefs_item.connect_activate_link */
    {
        let apps = appstate.clone();
        let prefs_item = &appstate.gui.popup_menu.prefs_item;
        prefs_item.connect_activate(move |_| {
                                        on_prefs_item_activate(&apps);
                                    });
    }

    /* reload_item.connect_activate_link */
    {
        let apps = appstate.clone();
        let reload_item = &appstate.gui.popup_menu.reload_item;
        reload_item.connect_activate(move |_| {
                                         try_w!(audio_reload(&apps.audio,
                                                 &apps.prefs.borrow(),
                                                 AudioUser::Popup))
                                     });
    }


    /* quit_item.connect_activate_link */
    {
        let quit_item = &appstate.gui.popup_menu.quit_item;
        quit_item.connect_activate(|_| { gtk::main_quit(); });
    }
}


/// When the about menu item is activated.
fn on_about_item_activate(appstate: &AppS) {
    let popup_menu = &appstate.gui.popup_menu.menu_window;
    let about_dialog = create_about_dialog();
    about_dialog.set_skip_taskbar_hint(true);
    about_dialog.set_transient_for(popup_menu);
    about_dialog.run();
    about_dialog.destroy();
}


/// Create the About dialog from scratch.
fn create_about_dialog() -> gtk::AboutDialog {
    let about_dialog: gtk::AboutDialog = gtk::AboutDialog::new();

    about_dialog.set_license(Some(
        "PNMixer-rs is free software; you can redistribute it and/or modify it
under the terms of the GNU General Public License v3 as published
by the Free Software Foundation.

PNMixer is distributed in the hope that it will be useful, but
WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.
See the GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with PNMixer; if not, write to the Free Software Foundation,
Inc., 51 Franklin Street, Fifth Floor, Boston, MA 02110-1301, USA.",
    ));
    about_dialog.set_copyright(Some("Copyright © 2017 Julian Ospald"));
    about_dialog.set_authors(&["Julian Ospald"]);
    about_dialog.set_artists(&["Paul Davey"]);
    about_dialog.set_program_name("PNMixer-rs");
    about_dialog.set_logo_icon_name("pnmixer");
    about_dialog.set_version(VERSION);
    about_dialog.set_website("https://github.com/hasufell/pnmixer-rust");
    about_dialog.set_comments("A mixer for the system tray");

    return about_dialog;
}


/// When the Preferences item is activated.
fn on_prefs_item_activate(appstate: &Rc<AppS>) {
    /* TODO: only create if needed */
    show_prefs_dialog(appstate);
}


/// When the Mute item is checked.
fn set_mute_check(apps: &Rc<AppS>) {
    let mute_check = &apps.gui.popup_menu.mute_check;
    let m_muted = apps.audio.get_mute();
    match m_muted {
        Ok(muted) => {
            mute_check.set_sensitive(false);
            mute_check.set_active(muted);
            mute_check.set_tooltip_text("");
        }
        Err(_) => {
            mute_check.set_active(true);
            mute_check.set_sensitive(false);
            mute_check.set_tooltip_text("Soundcard has no mute switch");
        }
    }
}
