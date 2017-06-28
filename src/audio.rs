extern crate alsa;
extern crate glib_sys;

use self::alsa::card::Card;
use self::alsa::mixer::{Mixer, Selem, Elem};
use alsa::mixer::SelemChannelId::*;
use std::iter::Map;
use libc::c_int;
use libc::c_uint;
use libc::c_void;
use errors::*;
use std::convert::From;
use libc::pollfd;
use app_state;
use std::cell::RefCell;



pub fn get_default_alsa_card() -> Card {
    return get_alsa_card_by_id(0);
}

pub fn get_alsa_card_by_id(index: c_int) -> Card {
    return alsa::Card::new(index);
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
    return Mixer::new(&format!("hw:{}", card.get_index()), false).cherr();
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
        let n = selem.get_id().get_name().map(|y| String::from(y))?;

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

pub fn get_vol(selem: &Selem) -> Result<f64> {
    let range = selem.get_playback_volume_range();
    let volume = selem.get_playback_volume(FrontRight).map(|v| {
        return vol_to_percent(v, range);
    });

    return volume.cherr();
}

pub fn set_vol(selem: &Selem, new_vol: f64) -> Result<()> {
    /* auto-unmute */
    if get_mute(selem)? {
        set_mute(selem, false)?;
    }

    let range = selem.get_playback_volume_range();
    selem.set_playback_volume_all(
        percent_to_vol(new_vol, range),
    )?;

    return Ok(());
}

pub fn has_mute(selem: &Selem) -> bool {
    return selem.has_playback_switch();
}

pub fn get_mute(selem: &Selem) -> Result<bool> {
    let val = selem.get_playback_switch(FrontRight)?;
    return Ok(val == 0);
}

pub fn set_mute(selem: &Selem, mute: bool) -> Result<()> {
    /* true -> mute, false -> unmute */
    let _ = selem.set_playback_switch_all(!mute as i32)?;
    return Ok(());
}



/* GIO */

pub fn watch_poll_descriptors(
    polls: Vec<pollfd>,
    acard: RefCell<app_state::AlsaCard>,
) -> Vec<c_uint> {
    let mut watch_ids: Vec<c_uint> = vec![];
    for poll in polls {
        unsafe {
            let gioc = glib_sys::g_io_channel_unix_new(poll.fd);
            watch_ids.push(glib_sys::g_io_add_watch(
                gioc,
                glib_sys::GIOCondition::from_bits_truncate(
                    glib_sys::G_IO_IN.bits() | glib_sys::G_IO_ERR.bits(),
                ),
                Some(watch_cb),
                acard.as_ptr() as glib_sys::gpointer,
            ));
        }
    }

    return vec![];
}

extern "C" fn watch_cb(
    chan: *mut glib_sys::GIOChannel,
    cond: glib_sys::GIOCondition,
    data: glib_sys::gpointer,
) -> glib_sys::gboolean {
    return true as glib_sys::gboolean;
}
