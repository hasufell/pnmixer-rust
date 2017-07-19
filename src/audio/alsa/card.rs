//! Alsa audio subsystem.
//!
//! This mod mainly defines the `AlsaCard` struct, which is the only data
//! structure interacting directly with the alsa library.
//! No other struct should directly interact with the alsa bindings.


use alsa_lib::card::Card;
use alsa_lib::mixer::{Mixer, Selem, SelemId};
use alsa_lib::poll::PollDescriptors;
use alsa_sys;
use errors::*;
use glib_sys;
use libc::c_uint;
use libc::pollfd;
use libc::size_t;
use std::cell::Cell;
use std::mem;
use std::ptr;
use std::rc::Rc;
use std::u8;
use support::alsa::*;



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
    pub fn new(
        card_name: Option<String>,
        elem_name: Option<String>,
        cb: Rc<Fn(AlsaEvent)>,
    ) -> Result<Box<AlsaCard>> {
        let card = {
            match card_name {
                Some(name) => {
                    if name == "(default)" {
                        let default = get_default_alsa_card();
                        if alsa_card_has_playable_selem(&default) {
                            default
                        } else {
                            warn!(
                                "Default alsa card not playabla, trying others"
                            );
                            get_first_playable_alsa_card()?
                        }
                    } else {
                        let mycard = get_alsa_card_by_name(name.clone());
                        match mycard {
                            Ok(card) => card,
                            Err(_) => {
                                warn!(
                                    "Card {} not playable, trying others",
                                    name
                                );
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
            let requested_selem = get_playable_selem_by_name(
                &mixer,
                elem_name.unwrap_or(String::from("Master")),
            );
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

        let watch_ids =
            AlsaCard::watch_poll_descriptors(vec_pollfd, acard.as_ref());
        acard.watch_ids.set(watch_ids);

        return Ok(acard);
    }

    /// Get the `Selem`, looked up by the `SelemId`.
    pub fn selem(&self) -> Selem {
        let selem_id = &self.selem_id;
        let selem = self.mixer.find_selem(selem_id);
        return selem.unwrap();
    }

    /// Watch the given alsa card poll descriptors and
    /// return the corresponding watch IDs for saving
    /// in the `AlsaCard` struct.
    fn watch_poll_descriptors(
        polls: Vec<pollfd>,
        acard: &AlsaCard,
    ) -> Vec<c_uint> {
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


/// The C callback function registered in `watch_poll_descriptors()`.
extern "C" fn watch_cb(
    chan: *mut glib_sys::GIOChannel,
    cond: glib_sys::GIOCondition,
    data: glib_sys::gpointer,
) -> glib_sys::gboolean {

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
