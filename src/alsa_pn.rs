use alsa::card::Card;
use alsa::mixer::SelemChannelId::*;
use alsa::mixer::{Mixer, Selem, Elem, SelemId};
use alsa::poll::PollDescriptors;
use alsa;
use alsa_sys;
use errors::*;
use glib_sys;
use libc::c_int;
use libc::c_uint;
use libc::pollfd;
use libc::size_t;
use std::cell::Cell;
use std::iter::Map;
use std::mem;
use std::ptr;
use std::rc::Rc;
use std::u8;



#[derive(Clone, Copy, Debug)]
pub enum AlsaEvent {
    AlsaCardError,
    AlsaCardDiconnected,
    AlsaCardValuesChanged,
}


pub struct AlsaCard {
    _cannot_construct: (),
    pub card: Card,
    pub mixer: Mixer,
    pub selem_id: SelemId,
    pub watch_ids: Cell<Vec<u32>>,
    pub cb: Rc<Fn(AlsaEvent)>,
}


impl AlsaCard {
    pub fn new(card_name: Option<String>,
               elem_name: Option<String>,
               cb: Rc<Fn(AlsaEvent)>)
               -> Result<Box<AlsaCard>> {
        let card = {
            match card_name {
                Some(name) => get_alsa_card_by_name(name)?,
                None => get_default_alsa_card(),
            }
        };
        let mixer = get_mixer(&card)?;

        let vec_pollfd = PollDescriptors::get(&mixer)?;

        let selem_id =
            get_selem_by_name(&mixer,
                              elem_name.unwrap_or(String::from("Master")))
                    .unwrap()
                    .get_id();

        let acard = Box::new(AlsaCard {
                                 _cannot_construct: (),
                                 card: card,
                                 mixer: mixer,
                                 selem_id: selem_id,
                                 watch_ids: Cell::new(vec![]),
                                 cb: cb,
                             });

        /* TODO: callback is registered here, which must be unregistered
         * when the card is destroyed!!
         * poll descriptors must be unwatched too */
        let watch_ids = AlsaCard::watch_poll_descriptors(vec_pollfd,
                                                         acard.as_ref());
        acard.watch_ids.set(watch_ids);

        return Ok(acard);
    }


    pub fn selem(&self) -> Selem {
        return get_selems(&self.mixer)
                   .nth(self.selem_id.get_index() as usize)
                   .unwrap();
    }


    pub fn get_vol(&self) -> Result<f64> {
        let selem = self.selem();
        let range = selem.get_playback_volume_range();
        let volume = selem.get_playback_volume(FrontRight).map(|v| {
            return vol_to_percent(v, range);
        });

        return volume.from_err();
    }


    pub fn set_vol(&self, new_vol: f64) -> Result<()> {
        let selem = self.selem();
        /* auto-unmute */
        if self.get_mute()? {
            self.set_mute(false)?;
        }

        let range = selem.get_playback_volume_range();
        selem.set_playback_volume_all(percent_to_vol(new_vol, range))?;

        return Ok(());
    }


    pub fn has_mute(&self) -> bool {
        let selem = self.selem();
        return selem.has_playback_switch();
    }


    pub fn get_mute(&self) -> Result<bool> {
        let selem = self.selem();
        let val = selem.get_playback_switch(FrontRight)?;
        return Ok(val == 0);
    }


    pub fn set_mute(&self, mute: bool) -> Result<()> {
        let selem = self.selem();
        /* true -> mute, false -> unmute */
        let _ = selem.set_playback_switch_all(!mute as i32)?;
        return Ok(());
    }


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


    fn unwatch_poll_descriptors(watch_ids: &Vec<u32>) {
        for watch_id in watch_ids {
            unsafe {
                glib_sys::g_source_remove(*watch_id);
            }
        }
    }
}


// TODO: test that this is actually triggered when switching cards
impl Drop for AlsaCard {
    // call Box::new(x), transmute the Box into a raw pointer, and then
    // std::mem::forget
    //
    // if you unregister the callback, you should keep a raw pointer to the
    // box
    //
    // For instance, `register` could return a raw pointer to the
    // Box + a std::marker::PhantomData with the appropriate
    // lifetime (if applicable)
    //
    // The struct could implement Drop, which unregisters the
    // callback and frees the Box, by simply transmuting the
    // raw pointer to a Box<T>
    fn drop(&mut self) {
        debug!("Destructing watch_ids: {:?}", self.watch_ids.get_mut());
        AlsaCard::unwatch_poll_descriptors(&self.watch_ids.get_mut());
    }
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
    let cb = &acard.cb;
    cb(AlsaEvent::AlsaCardValuesChanged);

    return true as glib_sys::gboolean;
}


pub fn get_default_alsa_card() -> Card {
    return get_alsa_card_by_id(0);
}


pub fn get_alsa_card_by_id(index: c_int) -> Card {
    return Card::new(index);
}


pub fn get_alsa_cards() -> alsa::card::Iter {
    return alsa::card::Iter::new();
}


pub fn get_alsa_card_by_name(name: String) -> Result<Card> {
    for r_card in get_alsa_cards() {
        let card = r_card?;
        let card_name = card.get_name()?;
        if name == card_name {
            return Ok(card);
        }
    }
    bail!("Not found a matching card named {}", name);
}


pub fn get_mixer(card: &Card) -> Result<Mixer> {
    return Mixer::new(&format!("hw:{}", card.get_index()), false).from_err();
}


pub fn get_selem(elem: Elem) -> Selem {
    /* in the ALSA API, there are currently only simple elements,
     * so this unwrap() should be safe.
     *http://www.alsa-project.org/alsa-doc/alsa-lib/group___mixer.html#enum-members */
    return Selem::new(elem).unwrap();
}


pub fn get_selems(mixer: &Mixer) -> Map<alsa::mixer::Iter, fn(Elem) -> Selem> {
    return mixer.iter().map(get_selem);
}


pub fn get_selem_by_name(mixer: &Mixer, name: String) -> Result<Selem> {
    for selem in get_selems(mixer) {
        let n = selem.get_id()
            .get_name()
            .map(|y| String::from(y))?;

        if n == name {
            return Ok(selem);
        }
    }
    bail!("Not found a matching selem named {}", name);
}


pub fn vol_to_percent(vol: i64, range: (i64, i64)) -> f64 {
    let (min, max) = range;
    return ((vol - min) as f64) / ((max - min) as f64) * 100.0;
}


pub fn percent_to_vol(vol: f64, range: (i64, i64)) -> i64 {
    let (min, max) = range;
    let _v = vol / 100.0 * ((max - min) as f64) + (min as f64);
    /* TODO: precision? Use direction. */
    return _v as i64;
}
