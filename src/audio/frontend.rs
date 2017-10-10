#![allow(missing_docs)] // enums

//! High-level audio subsystem.
//!
//! This is the middleman between the low-level audio backend (alsa),
//! and the high-level ui code.
//! This abstraction layer allows the high-level code to be completely unaware
//! of the underlying audio implementation, may it be alsa or whatever.


use errors::*;
use std::cell::Ref;
use std::cell::RefCell;
use std::f64;
use std::rc::Rc;
use support::audio::*;



#[derive(Clone, Copy, Debug)]
/// The volume level of the current audio configuration.
pub enum VolLevel {
    Muted,
    Low,
    Medium,
    High,
    Off,
}


/// An audio user, used to determine from where a signal originated.
#[derive(Clone, Copy, Debug)]
pub enum AudioUser {
    Unknown,
    Popup,
    TrayIcon,
    Hotkeys,
    PrefsWindow,
}


/// An audio signal. This will be used to connect callbacks to the
/// audio system and react appropriately.
#[derive(Clone, Copy, Debug)]
pub enum AudioSignal {
    NoCard,
    CardInitialized,
    CardCleanedUp,
    CardDisconnected,
    CardError,
    ValuesChanged,
}


#[derive(Clone)]
/// Convenience struct to make handling this madness easier.
pub struct Handlers {
    inner: Rc<RefCell<Vec<Box<Fn(AudioSignal, AudioUser)>>>>,
}


impl Handlers {
    pub fn new() -> Handlers {
        return Handlers { inner: Rc::new(RefCell::new(vec![])) };
    }


    pub fn borrow(&self) -> Ref<Vec<Box<Fn(AudioSignal, AudioUser)>>> {
        return self.inner.borrow();
    }


    pub fn add_handler(&self, cb: Box<Fn(AudioSignal, AudioUser)>) {
        self.inner.borrow_mut().push(cb);
    }
}


/// This is the audio frontend, which can be implemented by different backends,
/// e.g. Alsa or PulseAudio. The high-level UI code only calls these
/// functions, never the underlying backend functions. The backend
/// implementation must ensure proper state and consistency, especially
/// wrt handlers and switching the card.
pub trait AudioFrontend {
    /// Switches the current card. Must invoke handlers.
    /// ## `user`
    /// Where the card switch originates from.
    fn switch_card(
        &self,
        card_name: Option<String>,
        elem_name: Option<String>,
        user: AudioUser,
    ) -> Result<()>;

    /// Current volume. Between 0 and 100.
    /// This always gets the volume of the `FrontRight` channel, because that
    /// seems to be the safest bet.
    fn get_vol(&self) -> Result<f64>;

    /// Set the current volume. Must invoke handlers.
    /// ## `new_vol`
    /// Set the volume to this value. From 0 to 100.
    /// ## `user`
    /// Where the card switch originates from.
    /// ## `dir`
    /// The "direction" of the volume change, e.g. is it a decrease
    /// or increase. This helps with rounding problems.
    /// ## `auto_unmute`
    /// Whether to automatically unmute if the volume changes.
    fn set_vol(
        &self,
        new_vol: f64,
        user: AudioUser,
        dir: VolDir,
        auto_unmute: bool,
    ) -> Result<()>;

    /// Current volume level, nicely usable for e.g. selecting from a set
    /// of images.
    fn vol_level(&self) -> VolLevel;

    /// Increase the volume. The step to increasy by is taken from
    /// `self.scroll_step`.
    /// ## `user`
    /// Where the card switch originates from.
    fn increase_vol(&self, user: AudioUser, auto_unmute: bool) -> Result<()>;

    /// Decrease the volume. The step to decrease by is taken from
    /// `self.scroll_step`.
    /// ## `user`
    /// Where the card switch originates from.
    fn decrease_vol(&self, user: AudioUser, auto_unmute: bool) -> Result<()>;

    /// Whether the current audio configuration can be muted.
    fn has_mute(&self) -> bool;

    /// Get the mute state of the current audio configuration.
    fn get_mute(&self) -> Result<bool>;

    /// Set the mute state of the current audio configuration.
    /// Must invoke handlers.
    fn set_mute(&self, mute: bool, user: AudioUser) -> Result<()>;

    /// Toggle the mute state of the current audio configuration.
    fn toggle_mute(&self, user: AudioUser) -> Result<()>;

    /// Connect a signal handler to the audio subsystem. This can
    /// be done from anywhere, e.g. in the UI code to react to
    /// certain signals. Multiple handlers for the same signals are fine,
    /// they will be executed in order.
    fn connect_handler(&self, cb: Box<Fn(AudioSignal, AudioUser)>);

    /// Get the current card name.
    fn card_name(&self) -> Result<String>;

    /// Get the currently playable card names.
    fn playable_card_names(&self) -> Vec<String>;

    /// Get the currently playable channel names.
    fn playable_chan_names(&self, cardname: Option<String>) -> Vec<String>;

    /// Get the current active channel name.
    fn chan_name(&self) -> Result<String>;

    /// Set the scroll step.
    fn set_scroll_step(&self, scroll_step: u32);

    /// Get the scroll step.
    fn get_scroll_step(&self) -> u32;
}
