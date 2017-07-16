//! The hotkey subsystem.
//!
//! This file defines what's a hotkey and deals with the low-level XKBlib and
//! Gtk/Gdk.


use errors::*;
use gdk;
use gdk_sys;
use gdk_x11::*;
use gtk;
use x11;
use libc::c_int;
use libc::c_uint;


/// `xmodmap -pm`
/// List of key modifiers which will be ignored whenever
/// we check whether the defined hotkeys have been pressed.
lazy_static! {
    static ref KEY_MASKS: Vec<c_uint> = vec![
                gdk_sys::GdkModifierType::empty().bits(), // No Modkey
                gdk_sys::GDK_MOD2_MASK.bits(), // Numlock
                gdk_sys::GDK_LOCK_MASK.bits(), // Capslock
                gdk_sys::GDK_MOD2_MASK.bits() | gdk_sys::GDK_LOCK_MASK.bits(),
            ];
}


#[derive(Debug)]
/// A hotkey, described by the underlying gdk/X11 representation.
pub struct Hotkey {
    /// The hardware keycode.
    pub code: gdk::key,
    /// The modifier keys and mouse button that have been pressed
    /// in addition to the main key (e.g. Numlock/Capslock).
    /// This is the raw bit representation and can be converted to
    /// `gtk::ModifierType` via `self.mods()`.
    pub mod_bits: u32, // Makes serialize/deserialize easier
    /// X key symbol.
    pub sym: u64,
    /// Gtk Accelerator string.
    pub gtk_accel: String,
}

impl Hotkey {
    /// Creates a new hotkey and grabs it.
    pub fn new(code: gdk::key, mods: gdk::ModifierType) -> Result<Hotkey> {
        let display = gdk_x11_get_default_xdisplay();
        let mod_bits = mods.bits();
        let sym =
            unsafe { x11::xlib::XkbKeycodeToKeysym(display, code as u8, 0, 0) };
        let gtk_accel = gtk::accelerator_name(sym as u32, mods)
            .ok_or("Could net get accelerator name")?;

        let hotkey = Hotkey {
            code,
            mod_bits,
            sym,
            gtk_accel,
        };

        hotkey.grab()?;

        return Ok(hotkey);
    }

    /// Creates a new hotkey from an accelerator string and grabs it.
    pub fn new_from_accel(accel: &str) -> Result<Hotkey> {
        let (code, mods) = hotkey_accel_to_code(accel);
        return Hotkey::new(code, mods);
    }

    /// Grab a key manually. Should be paired with a ungrab() call.
    pub fn grab(&self) -> Result<()> {
        let display = gdk_x11_get_default_xdisplay();

        /* Init error handling */
        let old_hdlr = unsafe {
            GRAB_ERROR = 0;
            x11::xlib::XSetErrorHandler(Some(grab_error_handler))
        };

        /* Grab the key */
        for key in KEY_MASKS.iter() {
            unsafe {
                x11::xlib::XGrabKey(display,
                                    self.code,
                                    self.mod_bits | key,
                                    gdk_x11_get_default_root_xwindow(),
                                    1,
                                    x11::xlib::GrabModeAsync,
                                    x11::xlib::GrabModeAsync);
            }
        }

        /* Synchronize X */
        unsafe {
            x11::xlib::XFlush(display);
            x11::xlib::XSync(display, false as c_int);
        }

        /* Restore error handler */
        unsafe {
            x11::xlib::XSetErrorHandler(old_hdlr);
        }

        /* Check for error */
        unsafe {
            ensure!(GRAB_ERROR == 0, "Error grabbing");
        }

        return Ok(());
    }

    /// Ungrab a key manually. Should be paired with a grab() call.
    pub fn ungrab(&self) {
        let display = gdk_x11_get_default_xdisplay();

        for key in KEY_MASKS.iter() {
            unsafe {
                x11::xlib::XUngrabKey(display,
                                      self.code,
                                      self.mod_bits | key,
                                      gdk_x11_get_default_root_xwindow());
            }
        }
    }

    /// Checks if the keycode we got (minus modifiers like
    /// numlock/capslock) matches the hotkey.
    /// Thus numlock + o will match o.
    pub fn matches(&self, code: gdk::key, mods: gdk::ModifierType) -> bool {
        if code != self.code {
            return false;
        }

        for key in KEY_MASKS.iter() {
            if (self.mod_bits | key) == mods.bits() {
                return true;
            }
        }

        return false;
    }
}

impl Drop for Hotkey {
    fn drop(&mut self) {
        debug!("Ungrabbing hotkey");
        self.ungrab();
    }
}

/// Translate a Gtk Accelerator string to a key code and mods.
pub fn hotkey_accel_to_code(accel: &str) -> (gdk::key, gdk::ModifierType) {
    let display = gdk_x11_get_default_xdisplay();
    let (sym, mods) = gtk::accelerator_parse(accel);

    unsafe {
        if sym != 0 {
            return (x11::xlib::XKeysymToKeycode(display, sym as u64) as i32,
                    mods);
        } else {
            return (-1, mods);
        }
    }
}


static mut GRAB_ERROR: u8 = 0;

extern "C" fn grab_error_handler(_: *mut x11::xlib::Display,
                                 _: *mut x11::xlib::XErrorEvent)
                                 -> c_int {
    warn!("Error while grabbing hotkey");
    unsafe {
        GRAB_ERROR = 1;
    }
    return 0;
}
