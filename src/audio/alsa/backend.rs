#![allow(illegal_floating_point_literal_pattern)]

//! Alsa audio backend subsystem.
//!
//! This mod mainly defines the `AlsaBackend` struct.


use alsa_lib::mixer::SelemChannelId::*;
use audio::alsa::card::*;
use audio::frontend::*;
use errors::*;
use glib;
use std::cell::Cell;
use std::cell::RefCell;
use std::rc::Rc;
use support::alsa::*;
use support::audio::*;



/// Alsa implementation of the `AudioFrontend`.
pub struct AlsaBackend {
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


impl AlsaBackend {
    /// Creates the `AlsaBackend`, containing an `AlsaCard`
    /// and additional information.
    pub fn new(
        card_name: Option<String>,
        elem_name: Option<String>,
    ) -> Result<AlsaBackend> {


        let last_action_timestamp = Rc::new(RefCell::new(0));
        let handlers = Handlers::new();

        let cb = {
            let myhandler = handlers.clone();
            let ts = last_action_timestamp.clone();
            Rc::new(move |event| {
                on_alsa_event(&mut *ts.borrow_mut(), &myhandler.borrow(), event)
            })
        };

        let acard = AlsaCard::new(card_name, elem_name, cb);

        if acard.is_err() {
            invoke_handlers(
                &handlers.borrow(),
                AudioSignal::NoCard,
                AudioUser::Unknown,
            );
        } else {
            invoke_handlers(
                &handlers.borrow(),
                AudioSignal::CardInitialized,
                AudioUser::Unknown,
            );
        }

        let alsa_backend = AlsaBackend {
            _cannot_construct: (),
            acard: RefCell::new(acard?),
            last_action_timestamp: last_action_timestamp.clone(),
            handlers,
            scroll_step: Cell::new(5),
        };

        return Ok(alsa_backend);
    }


    /// Gets the volume range of the currently selected card configuration.
    ///
    /// # Returns
    ///
    /// `(min, max)`
    fn get_volume_range(&self) -> (i64, i64) {
        let acard = self.acard.borrow();
        let selem = acard.selem();
        return selem.get_playback_volume_range();
    }
}


impl AudioFrontend for AlsaBackend {
    fn switch_card(
        &self,
        card_name: Option<String>,
        elem_name: Option<String>,
        user: AudioUser,
    ) -> Result<()> {
        debug!("Switching cards");
        debug!("Old card name: {}", self.card_name().unwrap());
        debug!("Old chan name: {}", self.chan_name().unwrap());
        let cb = self.acard.borrow().cb.clone();

        {
            let mut ac = self.acard.borrow_mut();
            *ac = AlsaCard::new(card_name, elem_name, cb)?;
        }

        debug!("New card name: {}", self.card_name().unwrap());
        debug!("New chan name: {}", self.chan_name().unwrap());

        invoke_handlers(
            &self.handlers.borrow(),
            AudioSignal::CardInitialized,
            user,
        );

        return Ok(());
    }

    fn card_name(&self) -> Result<String> {
        return Ok(self.acard.borrow().card.get_name()?);
    }

    fn chan_name(&self) -> Result<String> {
        let n = self.acard.borrow().selem_id.get_name().map(
            |y| String::from(y),
        )?;
        return Ok(n);
    }

    fn playable_chan_names(&self) -> Vec<String> {
        return get_playable_selem_names(&self.acard.borrow().mixer);
    }

    fn get_vol(&self) -> Result<f64> {
        let acard = self.acard.borrow();
        let selem = acard.selem();
        let volume = selem.get_playback_volume(FrontRight)?;

        return vol_to_percent(volume, self.get_volume_range());
    }

    fn set_vol(
        &self,
        new_vol: f64,
        user: AudioUser,
        dir: VolDir,
        auto_unmute: bool,
    ) -> Result<()> {

        {
            let mut rc = self.last_action_timestamp.borrow_mut();
            *rc = glib::get_monotonic_time();
        }

        let alsa_vol = percent_to_vol(new_vol, self.get_volume_range(), dir)?;

        /* only invoke handlers etc. if volume did actually change */
        {
            let old_alsa_vol =
                percent_to_vol(self.get_vol()?, self.get_volume_range(), dir)?;

            if old_alsa_vol == alsa_vol {
                return Ok(());
            }
        }

        /* auto-unmute */
        if auto_unmute && self.has_mute() && self.get_mute()? {
            self.set_mute(false, user)?;
        }

        debug!(
            "Setting vol on card {:?} and chan {:?} to {:?} by user {:?}",
            self.card_name().unwrap(),
            self.chan_name().unwrap(),
            new_vol,
            user
        );


        self.acard.borrow().selem().set_playback_volume_all(
            alsa_vol,
        )?;

        invoke_handlers(
            &self.handlers.borrow(),
            AudioSignal::ValuesChanged,
            user,
        );
        return Ok(());

    }

    fn vol_level(&self) -> VolLevel {
        let muted = self.get_mute().unwrap_or(false);
        if muted {
            return VolLevel::Muted;
        }
        let cur_vol = try_r!(self.get_vol(), VolLevel::Muted);
        match cur_vol {
            0. => return VolLevel::Off,
            0.0...33.0 => return VolLevel::Low,
            0.0...66.0 => return VolLevel::Medium,
            0.0...100.0 => return VolLevel::High,
            _ => return VolLevel::Off,
        }
    }

    fn increase_vol(&self, user: AudioUser, auto_unmute: bool) -> Result<()> {
        let old_vol = self.get_vol()?;
        let new_vol = old_vol + (self.scroll_step.get() as f64);

        return self.set_vol(new_vol, user, VolDir::Up, auto_unmute);
    }

    fn decrease_vol(&self, user: AudioUser, auto_unmute: bool) -> Result<()> {
        let old_vol = self.get_vol()?;
        let new_vol = old_vol - (self.scroll_step.get() as f64);

        return self.set_vol(new_vol, user, VolDir::Down, auto_unmute);
    }

    fn has_mute(&self) -> bool {
        let acard = self.acard.borrow();
        let selem = acard.selem();
        return selem.has_playback_switch();
    }

    fn get_mute(&self) -> Result<bool> {
        let acard = self.acard.borrow();
        let selem = acard.selem();
        let val = selem.get_playback_switch(FrontRight)?;
        return Ok(val == 0);
    }

    fn set_mute(&self, mute: bool, user: AudioUser) -> Result<()> {
        let mut rc = self.last_action_timestamp.borrow_mut();
        *rc = glib::get_monotonic_time();

        debug!(
            "Setting mute to {} on card {:?} and chan {:?} by user {:?}",
            mute,
            self.card_name().unwrap(),
            self.chan_name().unwrap(),
            user
        );

        let acard = self.acard.borrow();
        let selem = acard.selem();
        /* true -> mute, false -> unmute */
        let _ = selem.set_playback_switch_all(!mute as i32)?;

        invoke_handlers(
            &self.handlers.borrow(),
            AudioSignal::ValuesChanged,
            user,
        );
        return Ok(());
    }

    fn toggle_mute(&self, user: AudioUser) -> Result<()> {
        let muted = self.get_mute()?;
        return self.set_mute(!muted, user);
    }

    fn connect_handler(&self, cb: Box<Fn(AudioSignal, AudioUser)>) {
        self.handlers.add_handler(cb);
    }

    fn set_scroll_step(&self, scroll_step: u32) {
        self.scroll_step.set(scroll_step);
    }

    fn get_scroll_step(&self) -> u32 {
        return self.scroll_step.get();
    }
}


/// Invokes the registered handlers.
fn invoke_handlers(
    handlers: &Vec<Box<Fn(AudioSignal, AudioUser)>>,
    signal: AudioSignal,
    user: AudioUser,
) {
    debug!(
        "Invoking handlers for signal {:?} by user {:?}",
        signal,
        user
    );
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
fn on_alsa_event(
    last_action_timestamp: &mut i64,
    handlers: &Vec<Box<Fn(AudioSignal, AudioUser)>>,
    alsa_event: AlsaEvent,
) {
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
            invoke_handlers(
                handlers,
                self::AudioSignal::CardError,
                self::AudioUser::Unknown,
            );
        }
        AlsaEvent::AlsaCardDiconnected => {
            invoke_handlers(
                handlers,
                self::AudioSignal::CardDisconnected,
                self::AudioUser::Unknown,
            );
        }
        AlsaEvent::AlsaCardValuesChanged => {
            invoke_handlers(
                handlers,
                self::AudioSignal::ValuesChanged,
                self::AudioUser::Unknown,
            );
        }
        AlsaEvent::AlsaCardReload => {
            invoke_handlers(
                handlers,
                self::AudioSignal::CardReload,
                self::AudioUser::Unknown,
            );
        }
    }
}
