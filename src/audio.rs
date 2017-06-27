extern crate alsa;
extern crate libc;

use self::alsa::card::Card;
use self::alsa::mixer::{Mixer, Selem, Elem};
use alsa::mixer::SelemChannelId::*;
use std::iter::Map;
use self::libc::c_int;
use errors::*;
use std::convert::From;



pub fn get_default_alsa_card() -> Card {
    return get_alsa_card_by_id(0);
}

pub fn get_alsa_card_by_id(index: c_int) -> Card {
    return alsa::Card::new(index);
}

pub fn get_alsa_cards() -> alsa::card::Iter {
    return alsa::card::Iter::new();
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
        let n = selem.get_id()
            .get_name()
            .map(|y| String::from(y))?;

        if n == name {
            return Ok(selem);
        }
    }
    bail!("Not found a matching selem named {}", name);
}

pub fn get_vol(selem: &Selem) -> Result<f64> {
    let (min, max) = selem.get_playback_volume_range();
    let volume = selem.get_playback_volume(FrontRight).map(|v| {
        return ((v - min) as f64) / ((max - min) as f64) * 100.0;
    });

    return volume.cherr();
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

