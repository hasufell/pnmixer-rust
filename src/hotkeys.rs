//! The hotkeys subsystem.
//!
//! This handles the PNMixer-rs specific hotkeys as a whole,
//! including communication with Xlib and intercepting key presses
//! before they can be interpreted by Gtk/Gdk.


use audio_frontend::*;
use errors::*;
use errors;
use gdk;
use gdk_sys;
use gdk_x11;
use glib::translate::*;
use glib_sys;
use hotkey::*;
use prefs::*;
use std::mem;
use std::rc::Rc;
use w_result::*;
use x11;



/// The possible Hotkeys for manipulating the volume.
pub struct Hotkeys<T>
    where T: AudioFrontend
{
    enabled: bool,
    mute_key: Option<Hotkey>,
    up_key: Option<Hotkey>,
    down_key: Option<Hotkey>,

    // need this to access audio in 'key_filter'
    audio: Rc<T>,
    auto_unmute: bool,
}

impl<T> Hotkeys<T>
    where T: AudioFrontend
{
    /// Creates the hotkeys subsystem and binds the hotkeys.
    pub fn new(prefs: &Prefs,
               audio: Rc<T>)
               -> WResult<Box<Hotkeys<T>>, errors::Error, errors::Error> {
        debug!("Creating hotkeys control");
        let mut hotkeys =
            Box::new(Hotkeys {
                         enabled: false,
                         mute_key: None,
                         up_key: None,
                         down_key: None,
                         audio: audio,
                         auto_unmute: prefs.behavior_prefs.unmute_on_vol_change,
                     });
        let mut warn = vec![];
        push_warning!(hotkeys.reload(prefs), warn);

        /* bind hotkeys */
        let data_ptr =
            unsafe {
                mem::transmute::<&Hotkeys<T>,
                                 glib_sys::gpointer>(hotkeys.as_ref())
            };
        hotkeys_add_filter(Some(key_filter::<T>), data_ptr);
        return WOk(hotkeys, warn);
    }

    /// Reload the Hotkeys from the preferences.
    /// If hotkeys are disabled, just sets all members to `None`.
    /// This has to be called each time the preferences are modified.
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, otherwise `Err(str)` if some of the hotkeys
    /// could not be grabbed, where `str` is a String that can be
    /// presented via e.g. `run_error_dialog()`.
    pub fn reload(&mut self, prefs: &Prefs) -> Result<()> {
        self.enabled = prefs.hotkey_prefs.enable_hotkeys;
        self.mute_key = None;
        self.up_key = None;
        self.down_key = None;

        /* Return if hotkeys are disabled */
        if self.enabled == false {
            return Ok(());
        }

        let hotkey_prefs = &prefs.hotkey_prefs;
        let new_hotkey = |keyname: &Option<String>| -> (Option<Hotkey>, bool) {
            match keyname {
                &Some(ref k) => {
                    let hotkey = Hotkey::new_from_accel(k.as_str());
                    if hotkey.as_ref().is_err() {
                        (None, true)
                    } else {
                        (Some(hotkey.unwrap()), false)
                    }
                }
                &None => (None, false), // no actual error, just no key
            }
        };

        /* Setup mute hotkey */
        let (m_unmute_key, mute_err) =
            new_hotkey(&hotkey_prefs.mute_unmute_key);
        if m_unmute_key.is_some() {
            self.mute_key = Some(m_unmute_key.unwrap());
        }

        /* Setup volume uphotkey */
        let (m_up_key, up_err) = new_hotkey(&hotkey_prefs.vol_up_key);
        if m_up_key.is_some() {
            self.up_key = Some(m_up_key.unwrap());
        }

        /* Setup volume down hotkey */
        let (m_down_key, down_err) = new_hotkey(&hotkey_prefs.vol_down_key);
        if m_down_key.is_some() {
            self.down_key = Some(m_down_key.unwrap());
        }

        if mute_err || up_err || down_err {
            bail!("Could not grab the following hotkeys:\n{}{}{}",
                  if mute_err { "(Mute/Unmute)\n" } else { "" },
                  if up_err { "(Volume Up)\n" } else { "" },
                  if down_err { "(Volume Down)\n" } else { "" },
                  );
        }

        return Ok(());
    }

    /// Bind hotkeys manually. Should be paired with an `unbind()` call.
    pub fn bind(&self) {
        debug!("Bind hotkeys");
        if self.mute_key.is_some() {
            if self.mute_key
                   .as_ref()
                   .unwrap()
                   .grab()
                   .is_err() {
                warn!("Could not grab mute key");
            };
        }
        if self.up_key.is_some() {
            if self.up_key
                   .as_ref()
                   .unwrap()
                   .grab()
                   .is_err() {
                warn!("Could not grab volume up key");
            };
        }
        if self.down_key.is_some() {
            if self.down_key
                   .as_ref()
                   .unwrap()
                   .grab()
                   .is_err() {
                warn!("Could not grab volume down key");
            };
        }

        let data_ptr =
            unsafe { mem::transmute::<&Hotkeys<T>, glib_sys::gpointer>(self) };
        hotkeys_add_filter(Some(key_filter::<T>), data_ptr);
    }

    /// Unbind hotkeys manually. Should be paired with a `bind()` call.
    pub fn unbind(&self) {
        debug!("Unbind hotkeys");
        if self.mute_key.is_some() {
            self.mute_key
                .as_ref()
                .unwrap()
                .ungrab();
        }
        if self.up_key.is_some() {
            self.up_key
                .as_ref()
                .unwrap()
                .ungrab();
        }
        if self.down_key.is_some() {
            self.down_key
                .as_ref()
                .unwrap()
                .ungrab();
        }

        let data_ptr =
            unsafe { mem::transmute::<&Hotkeys<T>, glib_sys::gpointer>(self) };
        hotkeys_remove_filter(Some(key_filter::<T>), data_ptr);
    }
}

impl<T> Drop for Hotkeys<T>
    where T: AudioFrontend
{
    fn drop(&mut self) {
        debug!("Freeing hotkeys");
        self.mute_key = None;
        self.up_key = None;
        self.down_key = None;

        let data_ptr = unsafe {
            mem::transmute::<&mut Hotkeys<T>, glib_sys::gpointer>(self)
        };

        hotkeys_remove_filter(Some(key_filter::<T>), data_ptr)
    }
}


/// Attaches the `key_filter()` function as a filter
/// to the root window, so it will intercept window events.
fn hotkeys_add_filter(function: gdk_sys::GdkFilterFunc,
                      data: glib_sys::gpointer) {
    // TODO: all the unwrapping :/
    let window = gdk_x11::gdk_x11_window_foreign_new_for_display(
        &mut gdk::Display::get_default().unwrap(),
        gdk_x11::gdk_x11_get_default_root_xwindow()
        ).unwrap();

    unsafe {
        gdk_sys::gdk_window_add_filter(window.to_glib_none().0, function, data);
    }
}


/// Removes the previously attached `key_filter()` function from
/// the root window.
fn hotkeys_remove_filter(function: gdk_sys::GdkFilterFunc,
                         data: glib_sys::gpointer) {
    // TODO: all the unwrapping :/
    let window = gdk_x11::gdk_x11_window_foreign_new_for_display(
        &mut gdk::Display::get_default().unwrap(),
        gdk_x11::gdk_x11_get_default_root_xwindow()
        ).unwrap();

    unsafe {
        gdk_sys::gdk_window_remove_filter(window.to_glib_none().0,
                                          function,
                                          data);
    }

}


/// This function is called before Gtk/Gdk can respond
/// to any(!) window event and handles pressed hotkeys.
extern "C" fn key_filter<T>(gdk_xevent: *mut gdk_sys::GdkXEvent,
                            _: *mut gdk_sys::GdkEvent,
                            data: glib_sys::gpointer)
                            -> gdk_sys::GdkFilterReturn
    where T: AudioFrontend
{
    let xevent = gdk_xevent as *mut x11::xlib::XKeyEvent;

    let hotkeys: &Hotkeys<T> =
        unsafe { mem::transmute::<glib_sys::gpointer, &Hotkeys<T>>(data) };
    let mute_key = &hotkeys.mute_key;
    let up_key = &hotkeys.up_key;
    let down_key = &hotkeys.down_key;
    let audio = &hotkeys.audio;

    let xevent_type = unsafe { (*xevent).type_ };
    if xevent_type == x11::xlib::KeyPress {
        return gdk_sys::GdkFilterReturn::Continue;
    }

    let xevent_key = unsafe { (*xevent).keycode };
    let xevent_state = unsafe { (*xevent).state };


    if mute_key.as_ref().is_some() &&
       mute_key.as_ref()
           .unwrap()
           .matches(xevent_key as i32,
                    gdk::ModifierType::from_bits(xevent_state).unwrap()) {
        just_warn!(audio.toggle_mute(AudioUser::Hotkeys));
    } else if up_key.as_ref().is_some() &&
              up_key.as_ref()
                  .unwrap()
                  .matches(xevent_key as i32,
                           gdk::ModifierType::from_bits(xevent_state)
                               .unwrap()) {
        just_warn!(audio.increase_vol(AudioUser::Hotkeys, hotkeys.auto_unmute));

    } else if down_key.as_ref().is_some() &&
              down_key.as_ref()
                  .unwrap()
                  .matches(xevent_key as i32,
                           gdk::ModifierType::from_bits(xevent_state)
                               .unwrap()) {
        just_warn!(audio.decrease_vol(AudioUser::Hotkeys, hotkeys.auto_unmute));
    }

    return gdk_sys::GdkFilterReturn::Continue;
}
