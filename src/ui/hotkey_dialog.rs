//! The ui hotkey preferences dialog.
//!
//! Usually run from the preferences window.


use errors::*;
use gdk::DeviceExt;
use gdk;
use gdk_sys;
use glib::translate::*;
use gtk::prelude::*;
use gtk;
use gtk_sys;
use libc::c_uint;
use std;



/// Hotkey dialog struct holding the relevant gtk widgets.
pub struct HotkeyDialog {
    hotkey_dialog: gtk::Dialog,
    // instruction_label: gtk::Label, // not needed
    key_pressed_label: gtk::Label,
}

impl HotkeyDialog {
    /// Creates a new hotkey dialog.
    pub fn new<P>(parent: &P, hotkey: String) -> HotkeyDialog
    where
        P: IsA<gtk::Window>,
    {
        let builder = gtk::Builder::new_from_string(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/data/ui/hotkey-dialog.glade"
        )));

        let hotkey_dialog: gtk::Dialog =
            builder.get_object("hotkey_dialog").unwrap();
        let instruction_label: gtk::Label =
            builder.get_object("instruction_label").unwrap();
        let key_pressed_label: gtk::Label =
            builder.get_object("key_pressed_label").unwrap();

        hotkey_dialog.set_title(format!("Set {} HotKey", hotkey).as_str());
        instruction_label.set_markup(
            format!("Press new HotKey for <b>{}</b>", hotkey)
                .as_str(),
        );

        hotkey_dialog.set_transient_for(parent);

        {
            let key_pressed_label = key_pressed_label.clone();
            hotkey_dialog.connect_key_press_event(move |_, e| {
                let mut state = e.get_state();

                unsafe {
                    let mut keyval: c_uint = 0;
                    let mut consumed: gdk_sys::GdkModifierType =
                        gdk_sys::GdkModifierType::empty();
                    gdk_sys::gdk_keymap_translate_keyboard_state(
                        gdk_sys::gdk_keymap_get_default(),
                        e.get_hardware_keycode() as u32,
                        state.to_glib(),
                        e.get_group() as i32,
                        &mut keyval as *mut c_uint,
                        std::ptr::null_mut(),
                        std::ptr::null_mut(),
                        &mut consumed as *mut gdk_sys::GdkModifierType,
                    );

                    let consumed: gdk::ModifierType = from_glib(!consumed);
                    state = state & consumed;
                    state = state & gtk::accelerator_get_default_mod_mask();

                    let key_text = gtk::accelerator_name(keyval, state);
                    key_pressed_label.set_text(
                        key_text
                            .unwrap_or(String::from("(None)"))
                            .as_str(),
                    );
                };
                return Inhibit(false);
            });
        }



        hotkey_dialog.connect_key_release_event(move |w, _| {
            w.response(gtk_sys::GtkResponseType::Ok as i32);
            return Inhibit(false);
        });

        return HotkeyDialog {
            hotkey_dialog,
            key_pressed_label,
        };
    }

    /// Runs the hotkey dialog and returns a String representing the hotkey
    /// that has been pressed.
    pub fn run(&self) -> Result<String> {
        self.hotkey_dialog.show_now();
        let device = gtk::get_current_event_device().ok_or(
            "Could not get current device",
        )?;
        let window = self.hotkey_dialog.get_window().ok_or(
            "Could not get window",
        )?;

        let m_grab_status = device.grab(
            &window,
            gdk::GrabOwnership::Application,
            true,
            gdk::KEY_PRESS_MASK,
            None,
            gdk_sys::GDK_CURRENT_TIME as u32,
        );

        if m_grab_status != gdk::GrabStatus::Success {
            bail!("Could not grab the keyboard");
        }

        let resp = self.hotkey_dialog.run();
        device.ungrab(gdk_sys::GDK_CURRENT_TIME as u32);

        if resp != gtk::ResponseType::Ok.into() {
            bail!(ErrorKind::GtkResponseCancel(
                String::from("not assigning hotkey"),
            ));
        }

        return Ok(self.key_pressed_label.get_text().ok_or(
            "Could not get text",
        )?);
    }
}


impl Drop for HotkeyDialog {
    fn drop(&mut self) {
        self.hotkey_dialog.destroy();
    }
}
