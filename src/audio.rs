use alsa::card::Card;
use alsa::mixer::{Mixer, Selem, SelemId};
use alsa::poll::PollDescriptors;
use alsa_sys;
use errors::*;
use glib_sys;
use libc::c_uint;
use libc::pollfd;
use libc::size_t;
use myalsa::*;
use std::mem;
use std::ptr;
use std::u8;



// TODO: implement free/destructor
pub struct AlsaCard {
    _cannot_construct: (),
    pub card: Card,
    pub mixer: Mixer,
    pub selem_id: SelemId,
    pub watch_ids: Vec<u32>,
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

        /* TODO: callback is registered here, which must be unregistered
         * when the mixer is destroyed!! */
        let watch_ids = watch_poll_descriptors(vec_pollfd, &mixer);

        return Ok(AlsaCard {
            _cannot_construct: (),
            card: card,
            mixer: mixer,
            selem_id: selem_id,
            watch_ids: watch_ids,
        });
    }


    pub fn selem(&self) -> Selem {
        return get_selems(&self.mixer)
            .nth(self.selem_id.get_index() as usize)
            .unwrap();
    }


    pub fn vol(&self) -> Result<f64> {
        return get_vol(&self.selem());
    }


    pub fn set_vol(&self, new_vol: f64) -> Result<()> {
        return set_vol(&self.selem(), new_vol);
    }


    pub fn has_mute(&self) -> bool {
        return has_mute(&self.selem());
    }


    pub fn get_mute(&self) -> Result<bool> {
        return get_mute(&self.selem());
    }


    pub fn set_mute(&self, mute: bool) -> Result<()> {
        return set_mute(&self.selem(), mute);
    }
}


pub enum AudioUser {
    AudioUserUnknown,
    AudioUserPopup,
    AudioUserTrayIcon,
    AudioUserHotkeys,
}


enum AudioSignal {
    AudioNoCard,
    AudioCardInitialized,
    AudioCardCleanedUp,
    AudioCardDisconnected,
    AudioCardError,
    AudioValuesChanged,
}


fn watch_poll_descriptors(
    polls: Vec<pollfd>,
    mixer: &Mixer,
) -> Vec<c_uint> {
    let mut watch_ids: Vec<c_uint> = vec![];
    let mixer_ptr = unsafe {
        mem::transmute::<&Mixer, &*mut alsa_sys::snd_mixer_t>(mixer)
    };
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
                *mixer_ptr as glib_sys::gpointer,
            ));
        }
    }

    return watch_ids;
}


extern fn watch_cb(
    chan: *mut glib_sys::GIOChannel,
    cond: glib_sys::GIOCondition,
    data: glib_sys::gpointer,
) -> glib_sys::gboolean {

    let mixer = data as *mut alsa_sys::snd_mixer_t;

    unsafe {
        alsa_sys::snd_mixer_handle_events(mixer);
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
                continue
            },
            glib_sys::G_IO_STATUS_NORMAL => println!("G_IO_STATUS_NORMAL"),
            glib_sys::G_IO_STATUS_ERROR => println!("G_IO_STATUS_ERROR"),
            glib_sys::G_IO_STATUS_EOF => println!("G_IO_STATUS_EOF"),
        }
        return true as glib_sys::gboolean;
    }

    return true as glib_sys::gboolean;
}

