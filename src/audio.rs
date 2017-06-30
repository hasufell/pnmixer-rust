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
use std::mem;
use std::cell::RefCell;
use std::cell::Ref;
use std::ptr;
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
    pub fn new(
        card_name: Option<String>,
        elem_name: Option<String>,
    ) -> Result<AlsaCard> {
        let card = {
            match card_name {
                Some(name) => get_alsa_card_by_name(name)?,
                None => get_default_alsa_card(),
            }
        };
        let mixer = get_mixer(&card)?;
        let selem_id = get_selem_by_name(
            &mixer,
            elem_name.unwrap_or(String::from("Master")),
        ).unwrap()
            .get_id();
        let vec_pollfd = PollDescriptors::get(&mixer)?;

        let mut acard = AlsaCard {
            _cannot_construct: (),
            card: card,
            mixer: mixer,
            selem_id: selem_id,
            watch_ids: vec![],
            last_action_timestamp: RefCell::new(0),
            handlers: RefCell::new(vec![]),
        };

        /* TODO: callback is registered here, which must be unregistered
         * when the mixer is destroyed!!
         * poll descriptors must be unwatched too */
        let watch_ids = watch_poll_descriptors(vec_pollfd, &acard);
        // acard.watch_ids = watch_ids;

        // println!("Watch IDs: {:?}", acard.watch_ids);
        println!("Last_Timestamp: {}", acard.last_action_timestamp.borrow());


        return Ok(acard);
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
            println!("glib::get_monotonic_time: {}", glib::get_real_time());
        }
        println!("Now timestamp: {}", self.last_action_timestamp.borrow());
        // TODO invoke handlers, make use of user
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
        return set_mute(&self.selem(), mute);
    }


    fn on_alsa_event(&self, alsa_event: AlsaEvent) {
        // let last: i64 = *Ref::clone(&self.last_action_timestamp.borrow());

        // if last != 0 {
            // let now: i64 = glib::get_monotonic_time();
            // let delay: i64 = now - last;
            // if delay < 1000000 {
                // println!("Too short: {} and {}", now, last);
                // println!("Delay: {}", delay);
                // return;
            // }
            // *self.last_action_timestamp.borrow_mut() = 0;
        // }


        /* external change */
        match alsa_event {
            // TODO: invoke handlers with AudioUserUnknown
            AlsaEvent::AlsaCardError => println!("AlsaCardError"),
            AlsaEvent::AlsaCardDiconnected => println!("AlsaCardDiconnected"),
            AlsaEvent::AlsaCardValuesChanged => {
                println!("AlsaCardValuesChanged");
                self.invoke_handlers(
                    self::AudioSignal::AudioValuesChanged,
                    self::AudioUser::AudioUserUnknown,
                );
            }
        }

    }


    fn invoke_handlers(&self, signal: AudioSignal, user: AudioUser) {

        let mut vec = vec![1,2,3];

        for v in &vec {
            println!("Elem: {}", v);
        }

        let handlers = self.handlers.borrow();
        let x: &Vec<Box<Fn(&AlsaCard, AudioSignal, AudioUser)>> = &*handlers;
        println!("Vec size: {}", handlers.capacity());
        // for handler in x {
            // // let unboxed = handler.as_ref();
            // // unboxed(&self, signal, user);
            // println!("Gogo");
        // }
    }


    pub fn connect_handler(
        &self,
        cb: Box<Fn(&AlsaCard, AudioSignal, AudioUser)>,
    ) {
        println!("Vec size before: {}", self.handlers.borrow().capacity());

        self.handlers.borrow_mut().push(cb);
        println!("Vec size after: {}", self.handlers.borrow().capacity());
    }
}


#[derive(Clone, Copy)]
pub enum AudioUser {
    AudioUserUnknown,
    AudioUserPopup,
    AudioUserTrayIcon,
    AudioUserHotkeys,
}


#[derive(Clone, Copy)]
pub enum AudioSignal {
    AudioNoCard,
    AudioCardInitialized,
    AudioCardCleanedUp,
    AudioCardDisconnected,
    AudioCardError,
    AudioValuesChanged,
}


#[derive(Clone, Copy)]
pub enum AlsaEvent {
    AlsaCardError,
    AlsaCardDiconnected,
    AlsaCardValuesChanged,
}


fn watch_poll_descriptors(polls: Vec<pollfd>, acard: &AlsaCard) -> Vec<c_uint> {
    let mut watch_ids: Vec<c_uint> = vec![];
    let acard_ptr =
        unsafe { mem::transmute::<&AlsaCard, &glib_sys::gpointer>(acard) };
    for poll in polls {
        unsafe {
            let gioc: *mut glib_sys::GIOChannel =
                glib_sys::g_io_channel_unix_new(poll.fd);
            watch_ids.push(glib_sys::g_io_add_watch(
                gioc,
                glib_sys::GIOCondition::from_bits(
                    glib_sys::G_IO_IN.bits() | glib_sys::G_IO_ERR.bits(),
                ).unwrap(),
                Some(watch_cb),
                *acard_ptr,
            ));
        }
    }

    println!("Handler size in watch_poll_descriptors: {}", acard.handlers.borrow().capacity());
    return watch_ids;
}


extern "C" fn watch_cb(
    chan: *mut glib_sys::GIOChannel,
    cond: glib_sys::GIOCondition,
    data: glib_sys::gpointer,
) -> glib_sys::gboolean {

    let acard =
        unsafe { mem::transmute::<&glib_sys::gpointer, &AlsaCard>(&data) };
    println!("Handler size in watch_cb: {}", acard.handlers.borrow().capacity());
    let mixer = unsafe {
        mem::transmute::<&Mixer, &*mut alsa_sys::snd_mixer_t>(&acard.mixer)
    };

    unsafe {
        alsa_sys::snd_mixer_handle_events(*mixer);
    }

    if cond == glib_sys::G_IO_ERR {
        return false as glib_sys::gboolean;
    }

    let mut sread: size_t = 1;
    let mut buf: Vec<u8> = vec![0; 256];

    while sread > 0 {
        let stat: glib_sys::GIOStatus = unsafe {
            glib_sys::g_io_channel_read_chars(
                chan,
                buf.as_mut_ptr() as *mut u8,
                256,
                &mut sread as *mut size_t,
                ptr::null_mut(),
            )
        };

        match stat {
            glib_sys::G_IO_STATUS_AGAIN => {
                println!("G_IO_STATUS_AGAIN");
                continue;
            }
            glib_sys::G_IO_STATUS_NORMAL => println!("G_IO_STATUS_NORMAL"),
            glib_sys::G_IO_STATUS_ERROR => println!("G_IO_STATUS_ERROR"),
            glib_sys::G_IO_STATUS_EOF => println!("G_IO_STATUS_EOF"),
        }
        return true as glib_sys::gboolean;
    }

    // TODO: handle alsa events, pass to 'on_alsa_event'

    println!("on_alsa_event triggering");
    acard.on_alsa_event(AlsaEvent::AlsaCardValuesChanged);

    return true as glib_sys::gboolean;
}
