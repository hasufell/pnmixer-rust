//! The popup window subsystem when the user left-clicks on the tray icon.
//!
//! This shows the manipulatable volume slider with the current volume and
//! the mute checkbox.


use app_state::*;
use audio::*;
use errors::*;
use gdk::DeviceExt;
use gdk::{GrabOwnership, GrabStatus, BUTTON_PRESS_MASK, KEY_PRESS_MASK};
use gdk;
use gdk_sys::{GDK_KEY_Escape, GDK_CURRENT_TIME};
use glib;
use gtk::ToggleButtonExt;
use gtk::prelude::*;
use gtk;
use prefs::*;
use std::cell::Cell;
use std::rc::Rc;
use support_audio::*;
use support_cmd::*;



/// The main struct for the popup window, holding all relevant sub-widgets
/// and some mutable state.
pub struct PopupWindow {
    _cant_construct: (),
    /// The main window for the popup window widget.
    pub popup_window: gtk::Window,
    /// The volume scale adjustment.
    pub vol_scale_adj: gtk::Adjustment,
    /// The volume scale.
    pub vol_scale: gtk::Scale,
    /// The mute check button.
    pub mute_check: gtk::CheckButton,
    /// The button to start the external mixer.
    pub mixer_button: gtk::Button,
    /// Signal for mute_check.connect_toggled callback,
    /// so we can block it temporarily.
    toggle_signal: Cell<u64>,
    /// Signal for vol_scale_adj.connect_value_changed callback,
    /// so we can block it temporarily.
    changed_signal: Cell<u64>,
}

impl PopupWindow {
    /// Constructor.
    pub fn new(builder: gtk::Builder) -> PopupWindow {
        return PopupWindow {
                   _cant_construct: (),
                   popup_window: builder.get_object("popup_window").unwrap(),
                   vol_scale_adj: builder.get_object("vol_scale_adj").unwrap(),
                   vol_scale: builder.get_object("vol_scale").unwrap(),
                   mute_check: builder.get_object("mute_check").unwrap(),
                   mixer_button: builder.get_object("mixer_button").unwrap(),
                   toggle_signal: Cell::new(0),
                   changed_signal: Cell::new(0),
               };
    }

    /// Update the popup window state, including the slider
    /// and the mute checkbutton.
    pub fn update(&self, audio: &Audio) -> Result<()> {
        let cur_vol = audio.vol()?;
        set_slider(&self.vol_scale_adj, cur_vol);

        self.update_mute_check(&audio);

        return Ok(());
    }

    /// Update the mute checkbutton.
    pub fn update_mute_check(&self, audio: &Audio) {
        let m_muted = audio.get_mute();

        glib::signal_handler_block(&self.mute_check, self.toggle_signal.get());

        match m_muted {
            Ok(val) => {
                self.mute_check.set_sensitive(true);
                self.mute_check.set_active(val);
                self.mute_check.set_tooltip_text("");
            }
            Err(_) => {
                /* can't figure out whether channel is muted, grey out */
                self.mute_check.set_active(true);
                self.mute_check.set_sensitive(false);
                self.mute_check.set_tooltip_text(
                    "Soundcard has no mute switch",
                );
            }
        }

        glib::signal_handler_unblock(&self.mute_check,
                                     self.toggle_signal.get());
    }

    /// Set the page increment fro the volume scale adjustment based on the
    /// preferences.
    fn set_vol_increment(&self, prefs: &Prefs) {
        self.vol_scale_adj
            .set_page_increment(prefs.behavior_prefs.vol_scroll_step);
        self.vol_scale_adj
            .set_step_increment(prefs.behavior_prefs.vol_fine_scroll_step);
    }
}


/// Initialize the popup window subsystem.
pub fn init_popup_window(appstate: Rc<AppS>) {
    /* audio.connect_handler */
    {
        let apps = appstate.clone();
        appstate.audio.connect_handler(Box::new(move |s, u| {
            /* skip if window is hidden */
            if !apps.gui
                    .popup_window
                    .popup_window
                    .get_visible() {
                return;
            }
            match (s, u) {
                /* Update only mute check here
                 * If the user changes the volume through the popup window,
                 * we MUST NOT update the slider value, it's been done already.
                 * It means that, as long as the popup window is visible,
                 * the slider value reflects the value set by user,
                 * and not the real value reported by the audio system.
                 */
                (_, AudioUser::Popup) => {
                    apps.gui.popup_window.update_mute_check(&apps.audio);
                }
                /* external change, safe to update slider too */
                (_, _) => {
                    try_w!(apps.gui.popup_window.update(&apps.audio));
                }
            }
        }));
    }

    /* mute_check.connect_toggled */
    {
        let _appstate = appstate.clone();
        let mute_check = &appstate.clone()
                              .gui
                              .popup_window
                              .mute_check;
        let toggle_signal =
            mute_check.connect_toggled(move |_| {
                                           on_mute_check_toggled(&_appstate)
                                       });
        appstate.gui
            .popup_window
            .toggle_signal
            .set(toggle_signal);
    }

    /* popup_window.connect_show */
    {
        let _appstate = appstate.clone();
        let popup_window = &appstate.clone()
                                .gui
                                .popup_window
                                .popup_window;
        popup_window.connect_show(move |_| on_popup_window_show(&_appstate));
    }

    /* vol_scale_adj.connect_value_changed */
    {
        let _appstate = appstate.clone();
        let vol_scale_adj = &appstate.clone()
                                 .gui
                                 .popup_window
                                 .vol_scale_adj;
        let changed_signal = vol_scale_adj.connect_value_changed(
            move |_| on_vol_scale_value_changed(&_appstate),
        );

        appstate.gui
            .popup_window
            .changed_signal
            .set(changed_signal);
    }

    /* popup_window.connect_event */
    {
        let popup_window = &appstate.clone()
                                .gui
                                .popup_window
                                .popup_window;
        popup_window.connect_event(move |w, e| on_popup_window_event(w, e));
    }

    /* mixer_button.connect_clicked */
    {
        let apps = appstate.clone();
        let mixer_button = &appstate.clone()
                                .gui
                                .popup_window
                                .mixer_button;
        mixer_button.connect_clicked(move |_| {
            apps.gui
                .popup_window
                .popup_window
                .hide();
            let _ = result_warn!(execute_vol_control_command(&apps.prefs.borrow()),
                Some(&apps.gui.popup_menu.menu_window));
        });
    }
}


/// When the popup window is shown.
fn on_popup_window_show(appstate: &AppS) {
    let popup_window = &appstate.gui.popup_window;
    appstate.gui.popup_window.set_vol_increment(&appstate.prefs.borrow());
    glib::signal_handler_block(&popup_window.vol_scale_adj,
                               popup_window.changed_signal.get());
    try_w!(appstate.gui.popup_window.update(&appstate.audio));
    glib::signal_handler_unblock(&popup_window.vol_scale_adj,
                                 popup_window.changed_signal.get());
    popup_window.vol_scale.grab_focus();
    try_w!(grab_devices(&appstate.gui.popup_window.popup_window));
}


/// On key or button press event on the popup window.
fn on_popup_window_event(w: &gtk::Window, e: &gdk::Event) -> gtk::Inhibit {
    match gdk::Event::get_event_type(e) {
        gdk::EventType::GrabBroken => w.hide(),
        gdk::EventType::KeyPress => {
            let key: gdk::EventKey = e.clone().downcast().unwrap();
            if key.get_keyval() == (GDK_KEY_Escape as u32) {
                w.hide();
            }
        }
        gdk::EventType::ButtonPress => {
            let device = try_wr!(
                gtk::get_current_event_device().ok_or(
                    "No current event device!",
                ),
                Inhibit(false)
            );
            let (window, _, _) =
                gdk::DeviceExt::get_window_at_position(&device);
            if window.is_none() {
                w.hide();
            }
        }
        _ => (),
    }

    return Inhibit(false);
}


/// When the volume scale slider is moved.
fn on_vol_scale_value_changed(appstate: &AppS) {
    let audio = &appstate.audio;
    let old_vol = try_w!(audio.vol());

    let val = appstate.gui
        .popup_window
        .vol_scale
        .get_value();

    let dir = vol_change_to_voldir(old_vol, val);

    try_w!(audio.set_vol(val,
                         AudioUser::Popup,
                         dir,
                         appstate.prefs
                             .borrow()
                             .behavior_prefs
                             .unmute_on_vol_change));
}


/// When the mute checkbutton is toggled.
fn on_mute_check_toggled(appstate: &AppS) {
    let audio = &appstate.audio;
    try_w!(audio.toggle_mute(AudioUser::Popup))
}


/// Set the volume slider to the given value.
pub fn set_slider(vol_scale_adj: &gtk::Adjustment, scale: f64) {
    vol_scale_adj.set_value(scale);
}


/// Grab all devices, keyboard and mouse.
fn grab_devices(window: &gtk::Window) -> Result<()> {
    let device = gtk::get_current_event_device().ok_or("No current device")?;

    let gdk_window = window.get_window().ok_or("No window?!")?;

    /* Grab the mouse */
    let m_grab_status =
        device.grab(&gdk_window,
                    GrabOwnership::None,
                    true,
                    BUTTON_PRESS_MASK,
                    None,
                    GDK_CURRENT_TIME as u32);

    if m_grab_status != GrabStatus::Success {
        warn!("Could not grab {}",
              device.get_name().unwrap_or(String::from("UNKNOWN DEVICE")));
    }

    /* Grab the keyboard */
    let k_dev = device.get_associated_device()
        .ok_or("Couldn't get associated device")?;

    let k_grab_status = k_dev.grab(&gdk_window,
                                   GrabOwnership::None,
                                   true,
                                   KEY_PRESS_MASK,
                                   None,
                                   GDK_CURRENT_TIME as u32);
    if k_grab_status != GrabStatus::Success {
        warn!("Could not grab {}",
              k_dev.get_name().unwrap_or(String::from("UNKNOWN DEVICE")));
    }

    return Ok(());
}
