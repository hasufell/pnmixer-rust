//! Alsa audio helper functions.
//!
//! This mod wraps around a few low-level alsa functions and abstracts
//! out the details we don't care about. Mainly used by the
//! [alsa_card mod](./alsa_card.html).


use alsa::card::Card;
use alsa::mixer::{Mixer, Selem, SelemId, Elem};
use alsa;
use errors::*;
use libc::c_int;
use std::iter::Map;
use std::iter::Filter;



pub fn get_default_alsa_card() -> Card {
    return get_alsa_card_by_id(0);
}


pub fn get_alsa_card_by_id(index: c_int) -> Card {
    return Card::new(index);
}


pub fn get_alsa_cards() -> alsa::card::Iter {
    return alsa::card::Iter::new();
}


pub fn get_first_playable_alsa_card() -> Result<Card> {
    for m_card in get_alsa_cards() {
        match m_card {
            Ok(card) => {
                if alsa_card_has_playable_selem(&card) {
                    return Ok(card);
                }
            }
            _ => (),
        }
    }

    bail!("No playable alsa card found!")
}


pub fn get_playable_alsa_card_names() -> Vec<String> {
    let mut vec = vec![];
    for m_card in get_alsa_cards() {
        match m_card {
            Ok(card) => {
                if alsa_card_has_playable_selem(&card) {
                    let m_name = card.get_name();
                    if m_name.is_ok() {
                        vec.push(m_name.unwrap())
                    }
                }
            }
            _ => (),
        }
    }

    return vec;
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


pub fn alsa_card_has_playable_selem(card: &Card) -> bool {
    let mixer = try_wr!(get_mixer(&card), false);
    for selem in get_playable_selems(&mixer) {
        if selem_is_playable(&selem) {
            return true;
        }
    }
    return false;
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


pub fn get_playable_selems(mixer: &Mixer) -> Vec<Selem> {
    let mut v = vec![];
    for s in mixer.iter().map(get_selem).filter(selem_is_playable) {
        v.push(s);
    }
    return v;
}


pub fn get_first_playable_selem(mixer: &Mixer) -> Result<Selem> {
    for s in mixer.iter().map(get_selem).filter(selem_is_playable) {
        return Ok(s);
    }

    bail!("No playable Selem found!")
}


pub fn get_playable_selem_names(mixer: &Mixer) -> Vec<String> {
    let mut vec = vec![];
    for selem in get_playable_selems(mixer) {
        let n = selem.get_id().get_name().map(|y| String::from(y));
        match n {
            Ok(name) => vec.push(name),
            _ => (),
        }
    }

    return vec;
}


pub fn get_playable_selem_by_name(mixer: &Mixer,
                                  name: String)
                                  -> Result<Selem> {
    for selem in get_playable_selems(mixer) {
        let n = selem.get_id()
            .get_name()
            .map(|y| String::from(y))?;

        if n == name {
            return Ok(selem);
        }
    }
    bail!("Not found a matching playable selem named {}", name);
}


pub fn selem_is_playable(selem: &Selem) -> bool {
    return selem.has_playback_volume();
}
