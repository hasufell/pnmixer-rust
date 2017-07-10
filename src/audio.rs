use alsa_card::*;
use errors::*;
use glib;
use std::cell::Cell;
use std::cell::Ref;
use std::cell::RefCell;
use std::f64;
use std::rc::Rc;



#[derive(Clone, Copy, Debug)]
pub enum VolLevel {
    Muted,
    Low,
    Medium,
    High,
    Off,
}


#[derive(Clone, Copy, Debug)]
pub enum AudioUser {
    Unknown,
    Popup,
    TrayIcon,
    Hotkeys,
    PrefsWindow,
}


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


pub struct Audio {
    _cannot_construct: (),
    pub acard: RefCell<Box<AlsaCard>>,
    pub last_action_timestamp: Rc<RefCell<i64>>,
    pub handlers: Handlers,
    pub scroll_step: Cell<u32>,
}


impl Audio {
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


    pub fn vol(&self) -> Result<f64> {
        return self.acard.borrow().get_vol();
    }


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


    pub fn set_vol(&self, new_vol: f64, user: AudioUser) -> Result<()> {
        {
            let mut rc = self.last_action_timestamp.borrow_mut();
            *rc = glib::get_monotonic_time();
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
            .set_vol(new_vol)?;

        invoke_handlers(&self.handlers.borrow(),
                        AudioSignal::ValuesChanged,
                        user);
        return Ok(());
    }


    pub fn increase_vol(&self, user: AudioUser) -> Result<()> {
        {
            let mut rc = self.last_action_timestamp.borrow_mut();
            *rc = glib::get_monotonic_time();
        }
        let old_vol = self.vol()?;
        let new_vol = f64::ceil(old_vol + (self.scroll_step.get() as f64));

        debug!("Increase vol on card {:?} and chan {:?} by {:?} to {:?}",
               self.acard
                   .borrow()
                   .card_name()
                   .unwrap(),
               self.acard
                   .borrow()
                   .chan_name()
                   .unwrap(),
               (new_vol - old_vol),
               new_vol);

        self.set_vol(new_vol, user)?;

        return Ok(());
    }


    pub fn decrease_vol(&self, user: AudioUser) -> Result<()> {
        {
            let mut rc = self.last_action_timestamp.borrow_mut();
            *rc = glib::get_monotonic_time();
        }
        let old_vol = self.vol()?;
        let new_vol = old_vol - (self.scroll_step.get() as f64);

        debug!("Decrease vol on card {:?} and chan {:?} by {:?} to {:?}",
               self.acard
                   .borrow()
                   .card_name()
                   .unwrap(),
               self.acard
                   .borrow()
                   .chan_name()
                   .unwrap(),
               (new_vol - old_vol),
               new_vol);

        self.set_vol(new_vol, user)?;

        return Ok(());
    }


    pub fn has_mute(&self) -> bool {
        return self.acard.borrow().has_mute();
    }


    pub fn get_mute(&self) -> Result<bool> {
        return self.acard.borrow().get_mute();
    }


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


    pub fn toggle_mute(&self, user: AudioUser) -> Result<()> {
        let muted = self.get_mute()?;
        return self.set_mute(!muted, user);
    }


    pub fn connect_handler(&self, cb: Box<Fn(AudioSignal, AudioUser)>) {
        self.handlers.add_handler(cb);
    }
}


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
        // TODO: invoke handlers with AudioUserUnknown
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
