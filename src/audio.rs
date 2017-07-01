use errors::*;
use glib;
use std::cell::RefCell;
use std::rc::Rc;
use std::f64;
use alsa_pn::*;



#[derive(Clone, Copy, Debug)]
pub enum AudioUser {
    AudioUserUnknown,
    AudioUserPopup,
    AudioUserTrayIcon,
    AudioUserHotkeys,
}


#[derive(Clone, Copy, Debug)]
pub enum AudioSignal {
    AudioNoCard,
    AudioCardInitialized,
    AudioCardCleanedUp,
    AudioCardDisconnected,
    AudioCardError,
    AudioValuesChanged,
}


pub struct Audio {
    _cannot_construct: (),
    pub acard: RefCell<Box<AlsaCard>>,
    pub last_action_timestamp: RefCell<i64>,
    pub handlers: Rc<RefCell<Vec<Box<Fn(AudioSignal, AudioUser)>>>>,
    pub scroll_step: RefCell<u32>,
}


impl Audio {
    pub fn new(card_name: Option<String>,
               elem_name: Option<String>)
               -> Result<Audio> {

        let handlers = Rc::new(RefCell::new(vec![]));
        let last_action_timestamp = RefCell::new(0);

        let myhandler = handlers.clone();
        let ts = last_action_timestamp.clone();
        let cb = Rc::new(move |event| {
                             Audio::on_alsa_event(&mut *ts.borrow_mut(),
                                                  &myhandler.borrow(),
                                                  event)
                         });

        let audio = Audio {
            _cannot_construct: (),
            acard: RefCell::new(AlsaCard::new(card_name, elem_name, cb)?),
            last_action_timestamp: last_action_timestamp.clone(),
            handlers: handlers.clone(),
            scroll_step: RefCell::new(5),
        };

        return Ok(audio);
    }


    pub fn switch_acard(&self,
                        card_name: Option<String>,
                        elem_name: Option<String>)
                        -> Result<()> {
        let mut ac = self.acard.borrow_mut();
        let cb = self.acard
            .borrow()
            .cb
            .clone();
        *ac = AlsaCard::new(card_name, elem_name, cb)?;

        return Ok(());
    }


    pub fn vol(&self) -> Result<f64> {
        return self.acard.borrow().get_vol();
    }


    pub fn set_vol(&self, new_vol: f64, user: AudioUser) -> Result<()> {
        {
            let mut rc = self.last_action_timestamp.borrow_mut();
            *rc = glib::get_monotonic_time();
        }
        // TODO invoke handlers, make use of user

        debug!("Setting vol to {:?} by user {:?}", new_vol, user);
        return self.acard.borrow().set_vol(new_vol);
    }


    pub fn increase_vol(&self, user: AudioUser) -> Result<()> {
        {
            let mut rc = self.last_action_timestamp.borrow_mut();
            *rc = glib::get_monotonic_time();
        }
        let old_vol = self.vol()?;
        let new_vol = f64::ceil(old_vol + (*self.scroll_step.borrow() as f64));

        debug!("Increase vol by {:?} to {:?}", (new_vol - old_vol), new_vol);

        return self.set_vol(new_vol, user);
    }


    pub fn decrease_vol(&self, user: AudioUser) -> Result<()> {
        {
            let mut rc = self.last_action_timestamp.borrow_mut();
            *rc = glib::get_monotonic_time();
        }
        let old_vol = self.vol()?;
        let new_vol = old_vol - (*self.scroll_step.borrow() as f64);

        debug!("Decrease vol by {:?} to {:?}", (new_vol - old_vol), new_vol);

        return self.set_vol(new_vol, user);
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
        // TODO invoke handlers, make use of user
        debug!("Setting mute to {} by user {:?}", mute, user);
        return self.acard.borrow().set_mute(mute);
    }


    pub fn connect_handler(&self, cb: Box<Fn(AudioSignal, AudioUser)>) {
        self.handlers.borrow_mut().push(cb);
    }


    fn invoke_handlers(handlers: &Vec<Box<Fn(AudioSignal, AudioUser)>>,
                       signal: AudioSignal,
                       user: AudioUser) {
        debug!("Invoking handlers for signal {:?} by user {:?}",
               signal,
               user);
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
            AlsaEvent::AlsaCardError => debug!("AlsaCardError"),
            AlsaEvent::AlsaCardDiconnected => debug!("AlsaCardDiconnected"),
            AlsaEvent::AlsaCardValuesChanged => {
                debug!("AlsaCardValuesChanged");
                Audio::invoke_handlers(handlers,
                                       self::AudioSignal::AudioValuesChanged,
                                       self::AudioUser::AudioUserUnknown);
            }
            e => warn!("Unhandled alsa event: {:?}", e),
        }

    }
}
