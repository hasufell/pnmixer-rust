use alsa::card::Card;
use alsa::mixer::{Mixer, Selem, SelemId};
use alsa::poll::PollDescriptors;
use alsa_sys;
use errors::*;
use glib;
use glib_sys;
use libc::c_uint;
use libc::pollfd;
use libc::size_t;
use myalsa::*;
use std::cell::Ref;
use std::cell::RefCell;
use std::mem;
use std::ptr;
use std::rc::Rc;
use std::u8;



// TODO: implement free/destructor
pub struct AlsaCard {
    _cannot_construct: (),
    pub card: Card,
    pub mixer: Mixer,
    pub selem_id: SelemId,
    pub watch_ids: Vec<u32>,
    pub last_action_timestamp: RefCell<i64>,
    pub handlers: RefCell<Vec<Box<Fn(&AlsaCard, AudioSignal, AudioUser)>>>,
}


/* TODO: AlsaCard cleanup */
impl AlsaCard {
    pub fn new(card_name: Option<String>,
               elem_name: Option<String>)
               -> Result<Rc<RefCell<AlsaCard>>> {
        let card = {
            match card_name {
                Some(name) => get_alsa_card_by_name(name)?,
                None => get_default_alsa_card(),
            }
        };
        let mixer = get_mixer(&card)?;
        let selem_id =
            get_selem_by_name(&mixer,
                              elem_name.unwrap_or(String::from("Master")))
                    .unwrap()
                    .get_id();
        let vec_pollfd = PollDescriptors::get(&mixer)?;

        let acard = Rc::new(RefCell::new(AlsaCard {
                                             _cannot_construct: (),
                                             card: card,
                                             mixer: mixer,
                                             selem_id: selem_id,
                                             watch_ids: vec![],
                                             last_action_timestamp:
                                                 RefCell::new(0),
                                             handlers: RefCell::new(vec![]),
                                         }));

        /* TODO: callback is registered here, which must be unregistered
         * when the mixer is destroyed!!
         * poll descriptors must be unwatched too */
        let watch_ids = watch_poll_descriptors(vec_pollfd,
                                               acard.clone().as_ptr());
        acard.borrow_mut().watch_ids = watch_ids;

        return Ok(acard.clone());
    }


    pub fn selem(&self) -> Selem {
        return get_selems(&self.mixer)
                   .nth(self.selem_id.get_index() as usize)
                   .unwrap();
    }


    pub fn vol(&self) -> Result<f64> {
        return get_vol(&self.selem());
    }


    pub fn set_vol(&self, new_vol: f64, user: AudioUser) -> Result<()> {
        {
            let mut rc = self.last_action_timestamp.borrow_mut();
            *rc = glib::get_monotonic_time();
        }
        // TODO invoke handlers, make use of user

        debug!("Setting vol to {:?} by user {:?}", new_vol, user);
        return set_vol(&self.selem(), new_vol);
    }


    pub fn has_mute(&self) -> bool {
        return has_mute(&self.selem());
    }


    pub fn get_mute(&self) -> Result<bool> {
        return get_mute(&self.selem());
    }


    pub fn set_mute(&self, mute: bool, user: AudioUser) -> Result<()> {
        let mut rc = self.last_action_timestamp.borrow_mut();
        *rc = glib::get_monotonic_time();
        // TODO invoke handlers, make use of user
        debug!("Setting mute to {} by user {:?}", mute, user);
        return set_mute(&self.selem(), mute);
    }


    fn on_alsa_event(&self, alsa_event: AlsaEvent) {
        let last: i64 = *Ref::clone(&self.last_action_timestamp.borrow());

        if last != 0 {
            let now: i64 = glib::get_monotonic_time();
            let delay: i64 = now - last;
            if delay < 1000000 {
                return;
            }
            debug!("Discarding last time stamp, too old");
            *self.last_action_timestamp.borrow_mut() = 0;
        }

        /* external change */
        match alsa_event {
            // TODO: invoke handlers with AudioUserUnknown
            AlsaEvent::AlsaCardError => debug!("AlsaCardError"),
            AlsaEvent::AlsaCardDiconnected => debug!("AlsaCardDiconnected"),
            AlsaEvent::AlsaCardValuesChanged => {
                debug!("AlsaCardValuesChanged");
                self.invoke_handlers(self::AudioSignal::AudioValuesChanged,
                                     self::AudioUser::AudioUserUnknown);
            }
            e => warn!("Unhandled alsa event: {:?}", e),
        }

    }


    fn invoke_handlers(&self, signal: AudioSignal, user: AudioUser) {
        debug!("Invoking handlers for signal {:?} by user {:?}",
               signal,
               user);
        let handlers = self.handlers.borrow();
        let x: &Vec<Box<Fn(&AlsaCard, AudioSignal, AudioUser)>> = &*handlers;
        for handler in x {
            let unboxed = handler.as_ref();
            unboxed(&self, signal, user);
        }
    }


    pub fn connect_handler(&self,
                           cb: Box<Fn(&AlsaCard, AudioSignal, AudioUser)>) {
        self.handlers.borrow_mut().push(cb);
    }
}


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


#[derive(Clone, Copy, Debug)]
pub enum AlsaEvent {
    AlsaCardError,
    AlsaCardDiconnected,
    AlsaCardValuesChanged,
}


fn watch_poll_descriptors(polls: Vec<pollfd>,
                          acard: *mut AlsaCard)
                          -> Vec<c_uint> {
    let mut watch_ids: Vec<c_uint> = vec![];
    let acard_ptr =
        unsafe { mem::transmute::<*mut AlsaCard, glib_sys::gpointer>(acard) };
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


extern "C" fn watch_cb(chan: *mut glib_sys::GIOChannel,
                       cond: glib_sys::GIOCondition,
                       data: glib_sys::gpointer)
                       -> glib_sys::gboolean {

    let acard =
        unsafe { mem::transmute::<glib_sys::gpointer, &AlsaCard>(data) };

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
            glib_sys::G_IO_STATUS_NORMAL => debug!("G_IO_STATUS_NORMAL"),
            glib_sys::G_IO_STATUS_ERROR => debug!("G_IO_STATUS_ERROR"),
            glib_sys::G_IO_STATUS_EOF => debug!("G_IO_STATUS_EOF"),
        }
        return true as glib_sys::gboolean;
    }

    acard.on_alsa_event(AlsaEvent::AlsaCardValuesChanged);

    return true as glib_sys::gboolean;
}
