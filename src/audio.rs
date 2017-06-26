extern crate alsa;

use self::alsa::card::{Card};
use self::alsa::mixer::{Mixer, Selem};



pub fn get_default_alsa_card() -> Card {
    let default_card: Card = Card::new(0);

    return default_card;
}

pub fn get_mixer(card: Card) -> Mixer {
    let mixer = Mixer::new(&format!("hw:{}", card.get_index()),
            false).unwrap();

    for elem in mixer.iter() {
        let selem: Selem = Selem::new(elem).unwrap();
        println!("Elem: {}", selem.get_id().get_name().unwrap());
    }

    return mixer;
}

pub fn get_channels(mixer: &Mixer) -> alsa::mixer::Iter {
    return mixer.iter();
}

pub fn get_channel_by_name<'a>(mixer: &'a Mixer, name: String) -> Option<Selem<'a>> {
    for elem in mixer.iter() {
        let m_selem = Selem::new(elem);
        let m_name = m_selem.and_then(|x| x.get_id().get_name().ok());
        let retval = m_name.map_or(false, |n| {
            return n == name;
        });

        if retval {
            return Selem::new(elem);
        }
    }

    return None;
}


// pub fn list_channels(card: Card, hctl: HCtl) -> [str] {

// }

