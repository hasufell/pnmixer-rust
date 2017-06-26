extern crate alsa;
extern crate std;
extern crate libc;

use self::alsa::card::{Card};
use self::alsa::mixer::{Mixer, Selem, Elem};
use alsa::mixer::SelemChannelId::*;
use std::iter::Map;
use self::libc::c_int;



pub fn get_default_alsa_card() -> Card {
    return get_alsa_card_by_id(0);
}

pub fn get_alsa_card_by_id(index: c_int) -> Card {
    return alsa::Card::new(index);
}

pub fn get_alsa_cards() -> alsa::card::Iter {
    return alsa::card::Iter::new();
}

pub fn get_mixer(card: Card) -> Mixer {
    let mixer = Mixer::new(&format!("hw:{}", card.get_index()),
            false).unwrap();

    return mixer;
}

pub fn get_selems(mixer: &Mixer) -> Map<alsa::mixer::Iter, fn(Elem) -> Selem> {
    return mixer.iter().map(get_selem);
}

pub fn get_selem_by_name<'a>(mixer: &'a Mixer, name: String) -> Option<Selem> {
    for selem in get_selems(mixer) {
        let m_name = selem.get_id().get_name().map(|y| String::from(y)).ok();
        let retval = m_name.map_or(false, |n| {
            return n == name;
        });

        if retval {
            return Some(selem);
        }
    }

    return None;
}

pub fn get_vol(selem: Selem) -> Result<f64, alsa::Error> {
    let (min, max) = selem.get_playback_volume_range();
    let volume = selem.get_playback_volume(FrontRight).map(|v| {
        return ((v - min) as f64) / ((max - min) as f64) * 100.0;
    });

    return volume;
}

pub fn get_selem(elem: Elem) -> Selem {
    /* in the ALSA API, there are currently only simple elements,
     * so this unwrap() should be safe.
     *http://www.alsa-project.org/alsa-doc/alsa-lib/group___mixer.html#enum-members */
    return Selem::new(elem).unwrap();
}

// pub fn list_channels(card: Card, hctl: HCtl) -> [str] {

// }

