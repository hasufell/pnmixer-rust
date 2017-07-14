#![allow(missing_docs)] // enums

//! High-level audio subsystem.
//!
//! This is the middleman between the low-level audio backend (alsa),
//! and the high-level ui code.
//! This abstraction layer allows the high-level code to be completely unaware
//! of the underlying audio implementation, may it be alsa or whatever.


use alsa_card::*;
use errors::*;
use glib;
use std::cell::Cell;
use std::cell::Ref;
use std::cell::RefCell;
use std::f64;
use std::rc::Rc;
use support_alsa::*;
use support_audio::*;



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
    fn new() -> Handlers {
        return Handlers { inner: Rc::new(RefCell::new(vec![])) };
    }


    fn borrow(&self) -> Ref<Vec<Box<Fn(AudioSignal, AudioUser)>>> {
        return self.inner.borrow();
    }


    fn add_handler(&self, cb: Box<Fn(AudioSignal, AudioUser)>) {
        self.inner.borrow_mut().push(cb);
    }
}


/// High-level Audio struct, which could theoretically be backend
/// agnostic.
pub struct Audio {
    _cannot_construct: (),
    /// The alsa card.
    pub acard: RefCell<Box<AlsaCard>>,
    /// Last timestamp of an internal action we triggered, e.g.
    /// by setting the volume or the mute state.
    pub last_action_timestamp: Rc<RefCell<i64>>,
    /// A set of handlers that react to audio signals. We can
    /// connect to these.
    pub handlers: Handlers,
    /// The step at which to increase/decrease the volume.
    /// This value is basically from the preferences.
    pub scroll_step: Cell<u32>,
}


impl Audio {
    /// Create a new Audio instance. This tries very hard to get
    /// a working configuration from the backend.
    /// ## `card_name`
    /// If a card name is provided, it will be tried. If `None` is provided
    /// or the given card name does not exist or is not playable, any other
    /// playable card is tried.
    /// ## `elem_name`
    /// If an elem name is provided, it will be tried. If `None` is provided
    /// or the given elem name does not exist or is not playable, any other
    /// playable elem is tried.
    ///
    /// # Returns
    ///
    /// `Ok(Audio)` on success, `Err(error)` otherwise.
    pub fn new(card_name: Option<String>,
               elem_name: Option<String>)
               -> Result<Audio> {

        let handlers = Handlers::new();
        let last_action_timestamp = Rc::new(RefCell::new(0));

        let cb = {
            let myhandler = handlers.clone();
            let ts = last_action_timestamp.clone();
            Rc::new(move |event| {
                        on_alsa_event(&mut *ts.borrow_mut(),
                                      &myhandler.borrow(),
                                      event)
                    })
        };

        let acard = AlsaCard::new(card_name, elem_name, cb);

        /* additionally dispatch signals */
        if acard.is_err() {
            invoke_handlers(&handlers.borrow(),
                            AudioSignal::NoCard,
                            AudioUser::Unknown);
        } else {
            invoke_handlers(&handlers.borrow(),
                            AudioSignal::CardInitialized,
                            AudioUser::Unknown);
        }

        let audio = Audio {
            _cannot_construct: (),
            acard: RefCell::new(acard?),
            last_action_timestamp: last_action_timestamp.clone(),
            handlers: handlers.clone(),
            scroll_step: Cell::new(5),
        };

        return Ok(audio);
    }


    /// Switches the current alsa card. Behaves the same way in regards to
    /// `card_name` and `elem_name` as the `Audio::new()` method.
    /// ## `user`
    /// Where the card switch originates from.
    pub fn switch_acard(&self,
                        card_name: Option<String>,
                        elem_name: Option<String>,
                        user: AudioUser)
                        -> Result<()> {
        debug!("Switching cards");
        debug!("Old card name: {}",
               self.acard
                   .borrow()
                   .card_name()
                   .unwrap());
        debug!("Old chan name: {}",
               self.acard
                   .borrow()
                   .chan_name()
                   .unwrap());
        let cb = self.acard
            .borrow()
            .cb
            .clone();
        {
            let mut ac = self.acard.borrow_mut();
            *ac = AlsaCard::new(card_name, elem_name, cb)?;
        }

        // invoke_handlers(&self.handlers.borrow(),
        // AudioSignal::CardCleanedUp,
        // user);
        invoke_handlers(&self.handlers.borrow(),
                        AudioSignal::CardInitialized,
                        user);

        return Ok(());
    }


    /// Current volume.
    pub fn vol(&self) -> Result<f64> {
        let alsa_vol = self.acard
            .borrow()
            .get_vol()?;
        return vol_to_percent(alsa_vol, self.acard.borrow().get_volume_range());
    }


    /// Current volume level, nicely usable for e.g. selecting from a set
    /// of images.
    pub fn vol_level(&self) -> VolLevel {
        let muted = self.get_mute().unwrap_or(false);
        if muted {
            return VolLevel::Muted;
        }
        let cur_vol = try_r!(self.vol(), VolLevel::Muted);
        match cur_vol {
            0. => return VolLevel::Off,
            0.0...33.0 => return VolLevel::Low,
            0.0...66.0 => return VolLevel::Medium,
            0.0...100.0 => return VolLevel::High,
            _ => return VolLevel::Off,
        }
    }


    /// Set the current volume.
    /// ## `new_vol`
    /// Set the volume to this value.
    /// ## `user`
    /// Where the card switch originates from.
    /// ## `dir`
    /// The "direction" of the volume change, e.g. is it a decrease
    /// or increase. This helps with rounding problems.
    /// ## `auto_unmute`
    /// Whether to automatically unmute if the volume changes.
    pub fn set_vol(&self,
                   new_vol: f64,
                   user: AudioUser,
                   dir: VolDir,
                   auto_unmute: bool)
                   -> Result<()> {
        {
            let mut rc = self.last_action_timestamp.borrow_mut();
            *rc = glib::get_monotonic_time();
        }

        let alsa_vol = percent_to_vol(new_vol,
                                      self.acard.borrow().get_volume_range(),
                                      dir)?;

        /* only invoke handlers etc. if volume did actually change */
        {
            let old_alsa_vol =
                percent_to_vol(self.vol()?,
                               self.acard.borrow().get_volume_range(),
                               dir)?;

            if old_alsa_vol == alsa_vol {
                return Ok(());
            }
        }

        /* auto-unmute */
        if auto_unmute && self.has_mute() && self.get_mute()? {
            self.set_mute(false, user)?;
        }

        debug!("Setting vol on card {:?} and chan {:?} to {:?} by user {:?}",
               self.acard
                   .borrow()
                   .card_name()
                   .unwrap(),
               self.acard
                   .borrow()
                   .chan_name()
                   .unwrap(),
               new_vol,
               user);

        self.acard
            .borrow()
            .set_vol(alsa_vol)?;

        invoke_handlers(&self.handlers.borrow(),
                        AudioSignal::ValuesChanged,
                        user);
        return Ok(());
    }


    /// Increase the volume. The step to increasy by is taken from
    /// `self.scroll_step`.
    /// ## `user`
    /// Where the card switch originates from.
    pub fn increase_vol(&self,
                        user: AudioUser,
                        auto_unmute: bool)
                        -> Result<()> {
        let old_vol = self.vol()?;
        let new_vol = old_vol + (self.scroll_step.get() as f64);

        return self.set_vol(new_vol, user, VolDir::Up, auto_unmute);
    }


    /// Decrease the volume. The step to decrease by is taken from
    /// `self.scroll_step`.
    /// ## `user`
    /// Where the card switch originates from.
    pub fn decrease_vol(&self,
                        user: AudioUser,
                        auto_unmute: bool)
                        -> Result<()> {
        let old_vol = self.vol()?;
        let new_vol = old_vol - (self.scroll_step.get() as f64);

        return self.set_vol(new_vol, user, VolDir::Down, auto_unmute);
    }


    /// Whether the current audio configuration can be muted.
    pub fn has_mute(&self) -> bool {
        return self.acard.borrow().has_mute();
    }


    /// Get the mute state of the current audio configuration.
    pub fn get_mute(&self) -> Result<bool> {
        return self.acard.borrow().get_mute();
    }


    /// Set the mute state of the current audio configuration.
    pub fn set_mute(&self, mute: bool, user: AudioUser) -> Result<()> {
        let mut rc = self.last_action_timestamp.borrow_mut();
        *rc = glib::get_monotonic_time();

        debug!("Setting mute to {} on card {:?} and chan {:?} by user {:?}",
               mute,
               self.acard
                   .borrow()
                   .card_name()
                   .unwrap(),
               self.acard
                   .borrow()
                   .chan_name()
                   .unwrap(),
               user);

        self.acard
            .borrow()
            .set_mute(mute)?;

        invoke_handlers(&self.handlers.borrow(),
                        AudioSignal::ValuesChanged,
                        user);
        return Ok(());
    }


    /// Toggle the mute state of the current audio configuration.
    pub fn toggle_mute(&self, user: AudioUser) -> Result<()> {
        let muted = self.get_mute()?;
        return self.set_mute(!muted, user);
    }


    /// Connect a signal handler to the audio subsystem. This can
    /// be done from anywhere, e.g. in the UI code to react to
    /// certain signals. Multiple handlers for the same signals are fine,
    /// they will be executed in order.
    pub fn connect_handler(&self, cb: Box<Fn(AudioSignal, AudioUser)>) {
        self.handlers.add_handler(cb);
    }
}


/// Invokes the registered handlers.
fn invoke_handlers(handlers: &Vec<Box<Fn(AudioSignal, AudioUser)>>,
                   signal: AudioSignal,
                   user: AudioUser) {
    debug!("Invoking handlers for signal {:?} by user {:?}",
           signal,
           user);
    if handlers.is_empty() {
        debug!("No handler found");
    } else {
        debug!("Executing handlers")
    }
    for handler in handlers {
        let unboxed = handler.as_ref();
        unboxed(signal, user);
    }
}


/// The callback for alsa events that is passed to the alsa subsystem.
/// This is the bridge between low-level alsa events and "high-level"
/// audio system signals.
fn on_alsa_event(last_action_timestamp: &mut i64,
                 handlers: &Vec<Box<Fn(AudioSignal, AudioUser)>>,
                 alsa_event: AlsaEvent) {
    let last: i64 = *last_action_timestamp;

    if last != 0 {
        let now: i64 = glib::get_monotonic_time();
        let delay: i64 = now - last;
        if delay < 1000000 {
            return;
        }
        debug!("Discarding last time stamp, too old");
        *last_action_timestamp = 0;
    }

    /* external change */
    match alsa_event {
        AlsaEvent::AlsaCardError => {
            invoke_handlers(handlers,
                            self::AudioSignal::CardError,
                            self::AudioUser::Unknown);
        }
        AlsaEvent::AlsaCardDiconnected => {
            invoke_handlers(handlers,
                            self::AudioSignal::CardDisconnected,
                            self::AudioUser::Unknown);
        }
        AlsaEvent::AlsaCardValuesChanged => {
            invoke_handlers(handlers,
                            self::AudioSignal::ValuesChanged,
                            self::AudioUser::Unknown);
        }
        e => warn!("Unhandled alsa event: {:?}", e),
    }

}
