#![allow(illegal_floating_point_literal_pattern)]

//! Alsa audio subsystem.
//!
//! This mod mainly defines the `AlsaCard` struct, which is the only data
//! structure interacting directly with the alsa library.
//! No other struct should directly interact with the alsa bindings.


use alsa::card::Card;
use alsa::mixer::SelemChannelId::*;
use alsa::mixer::{Mixer, Selem, SelemId};
use alsa::poll::PollDescriptors;
use alsa_sys;
use audio::*;
use errors::*;
use glib;
use glib_sys;
use libc::c_uint;
use libc::pollfd;
use libc::size_t;
use std::cell::Cell;
use std::cell::RefCell;
use std::mem;
use std::ptr;
use std::rc::Rc;
use std::u8;
use support_alsa::*;
use support_audio::*;



#[derive(Clone, Copy, Debug)]
/// An "external" alsa card event, potentially triggered by anything.
pub enum AlsaEvent {
    /// An error.
    AlsaCardError,
    /// Alsa card is disconnected.
    AlsaCardDiconnected,
    /// The values of the mixer changed, including mute state.
    AlsaCardValuesChanged,
}


/// A fairly high-level alsa card struct. We save some redundant
/// information in order to access it more easily, in addition to
/// some information that is not purely alsa related (like callbacks).
pub struct AlsaCard {
    _cannot_construct: (),
    /// The raw alsa card.
    pub card: Card,
    /// The raw mixer.
    pub mixer: Mixer,
    /// The simple element ID. `Selem` doesn't implement the Copy trait
    /// so we save the ID instead and can get the `Selem` by lookup.
    pub selem_id: SelemId,
    /// Watch IDs from polling the alsa card. We need them when we
    /// drop the card, so we can unregister the polling.
    pub watch_ids: Cell<Vec<u32>>,
    /// Callback for the various `AlsaEvent`s.
    pub cb: Rc<Fn(AlsaEvent)>,
}


impl AlsaCard {
    /// Create a new alsa card. Tries very hard to get a valid, playable
    /// card and mixer, so this is not a 'strict' function.
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
    /// `Ok(Box<AlsaCard>)` on success, `Err(error)` otherwise.
    pub fn new(card_name: Option<String>,
               elem_name: Option<String>,
               cb: Rc<Fn(AlsaEvent)>)
               -> Result<Box<AlsaCard>> {
        let card = {
            match card_name {
                Some(name) => {
                    if name == "(default)" {
                        let default = get_default_alsa_card();
                        if alsa_card_has_playable_selem(&default) {
                            default
                        } else {
                            warn!("Default alsa card not playabla, trying others");
                            get_first_playable_alsa_card()?
                        }
                    } else {
                        let mycard = get_alsa_card_by_name(name.clone());
                        match mycard {
                            Ok(card) => card,
                            Err(_) => {
                                warn!("Card {} not playable, trying others",
                                      name);
                                get_first_playable_alsa_card()?
                            }
                        }
                    }
                }
                None => get_first_playable_alsa_card()?,
            }
        };
        let mixer = get_mixer(&card)?;

        let selem_id = {
            let requested_selem =
                get_playable_selem_by_name(&mixer,
                                           elem_name.unwrap_or(String::from("Master")));
            match requested_selem {
                Ok(s) => s.get_id(),
                Err(_) => {
                    warn!("No playable Selem found, trying others");
                    get_first_playable_selem(&mixer)?.get_id()
                }
            }
        };

        let vec_pollfd = PollDescriptors::get(&mixer)?;

        let acard = Box::new(AlsaCard {
                                 _cannot_construct: (),
                                 card,
                                 mixer,
                                 selem_id,
                                 watch_ids: Cell::new(vec![]),
                                 cb,
                             });

        let watch_ids = AlsaCard::watch_poll_descriptors(vec_pollfd,
                                                         acard.as_ref());
        acard.watch_ids.set(watch_ids);

        return Ok(acard);
    }

    /// Get the `Selem`, looked up by the `SelemId`.
    fn selem(&self) -> Selem {
        let selem_id = &self.selem_id;
        let selem = self.mixer.find_selem(selem_id);
        return selem.unwrap();
    }

    /// Watch the given alsa card poll descriptors and
    /// return the corresponding watch IDs for saving
    /// in the `AlsaCard` struct.
    fn watch_poll_descriptors(polls: Vec<pollfd>,
                              acard: &AlsaCard)
                              -> Vec<c_uint> {
        let mut watch_ids: Vec<c_uint> = vec![];
        let acard_ptr =
            unsafe { mem::transmute::<&AlsaCard, glib_sys::gpointer>(acard) };
        for poll in polls {
            let gioc: *mut glib_sys::GIOChannel =
                unsafe { glib_sys::g_io_channel_unix_new(poll.fd) };
            let id = unsafe {
                glib_sys::g_io_add_watch(
                    gioc,
                    glib_sys::GIOCondition::from_bits(
                        glib_sys::G_IO_IN.bits() | glib_sys::G_IO_ERR.bits(),
                    ).unwrap(),
                    Some(watch_cb),
                    acard_ptr,
                )
            };
            watch_ids.push(id);
            unsafe { glib_sys::g_io_channel_unref(gioc) }
        }

        return watch_ids;
    }

    /// Unwatch the given poll descriptors.
    fn unwatch_poll_descriptors(watch_ids: &Vec<u32>) {
        for watch_id in watch_ids {
            unsafe {
                glib_sys::g_source_remove(*watch_id);
            }
        }
    }
}


impl Drop for AlsaCard {
    /// Destructs the watch IDs corresponding to the current poll descriptors.
    fn drop(&mut self) {
        debug!("Destructing watch_ids: {:?}", self.watch_ids.get_mut());
        AlsaCard::unwatch_poll_descriptors(&self.watch_ids.get_mut());
    }
}


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
    pub fn new(card_name: Option<String>,
               elem_name: Option<String>)
               -> Result<AlsaBackend> {


        let last_action_timestamp = Rc::new(RefCell::new(0));
        let handlers = Handlers::new();

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

        if acard.is_err() {
            invoke_handlers(&handlers.borrow(),
                            AudioSignal::NoCard,
                            AudioUser::Unknown);
        } else {
            invoke_handlers(&handlers.borrow(),
                            AudioSignal::CardInitialized,
                            AudioUser::Unknown);
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
    fn switch_card(&self,
                   card_name: Option<String>,
                   elem_name: Option<String>,
                   user: AudioUser)
                   -> Result<()> {
        debug!("Switching cards");
        debug!("Old card name: {}", self.card_name().unwrap());
        debug!("Old chan name: {}", self.chan_name().unwrap());
        let cb = self.acard
            .borrow()
            .cb
            .clone();

        {
            let mut ac = self.acard.borrow_mut();
            *ac = AlsaCard::new(card_name, elem_name, cb)?;
        }

        invoke_handlers(&self.handlers.borrow(),
                        AudioSignal::CardInitialized,
                        user);

        return Ok(());
    }

    fn card_name(&self) -> Result<String> {
        return self.acard
                   .borrow()
                   .card
                   .get_name()
                   .from_err();
    }

    fn chan_name(&self) -> Result<String> {
        let n = self.acard
            .borrow()
            .selem_id
            .get_name()
            .map(|y| String::from(y))?;
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

    fn set_vol(&self,
               new_vol: f64,
               user: AudioUser,
               dir: VolDir,
               auto_unmute: bool)
               -> Result<()> {

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

        debug!("Setting vol on card {:?} and chan {:?} to {:?} by user {:?}",
               self.card_name().unwrap(),
               self.chan_name().unwrap(),
               new_vol,
               user);


        self.acard
            .borrow()
            .selem()
            .set_playback_volume_all(alsa_vol)?;

        invoke_handlers(&self.handlers.borrow(),
                        AudioSignal::ValuesChanged,
                        user);
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

        debug!("Setting mute to {} on card {:?} and chan {:?} by user {:?}",
               mute,
               self.card_name().unwrap(),
               self.chan_name().unwrap(),
               user);

        let acard = self.acard.borrow();
        let selem = acard.selem();
        /* true -> mute, false -> unmute */
        let _ = selem.set_playback_switch_all(!mute as i32)?;

        invoke_handlers(&self.handlers.borrow(),
                        AudioSignal::ValuesChanged,
                        user);
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





/// The C callback function registered in `watch_poll_descriptors()`.
extern "C" fn watch_cb(chan: *mut glib_sys::GIOChannel,
                       cond: glib_sys::GIOCondition,
                       data: glib_sys::gpointer)
                       -> glib_sys::gboolean {

    let acard =
        unsafe { mem::transmute::<glib_sys::gpointer, &AlsaCard>(data) };
    let cb = &acard.cb;

    unsafe {
        let mixer_ptr =
            mem::transmute::<&Mixer, &*mut alsa_sys::snd_mixer_t>(&acard.mixer);
        alsa_sys::snd_mixer_handle_events(*mixer_ptr);
    };

    if cond == glib_sys::G_IO_ERR {
        return false as glib_sys::gboolean;
    }

    let mut sread: size_t = 1;
    let mut buf: Vec<u8> = vec![0; 256];

    while sread > 0 {
        let stat: glib_sys::GIOStatus =
            unsafe {
                glib_sys::g_io_channel_read_chars(chan,
                                                  buf.as_mut_ptr() as *mut u8,
                                                  256,
                                                  &mut sread as *mut size_t,
                                                  ptr::null_mut())
            };

        match stat {
            glib_sys::G_IO_STATUS_AGAIN => {
                debug!("G_IO_STATUS_AGAIN");
                continue;
            }
            glib_sys::G_IO_STATUS_NORMAL => {
                error!("Alsa failed to clear the channel");
                cb(AlsaEvent::AlsaCardError);
            }
            glib_sys::G_IO_STATUS_ERROR => (),
            glib_sys::G_IO_STATUS_EOF => {
                error!("GIO error has occurred");
                cb(AlsaEvent::AlsaCardError);
            }
        }
        return true as glib_sys::gboolean;
    }
    cb(AlsaEvent::AlsaCardValuesChanged);

    return true as glib_sys::gboolean;
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
    }
}
