use gtk;

use alsa::card::Card;
use alsa::mixer::{Mixer, SelemId, Selem};
use audio;
use errors::*;

pub struct AppS {
    /* we keep this to ensure the lifetime is across the whole application */
    pub status_icon: gtk::StatusIcon,

    pub builder_popup: gtk::Builder,
}

pub struct AlsaCard {
    card: Card,
    mixer: Mixer,
    selem_id: SelemId,
}

impl AlsaCard {
    pub fn new(
        card_name: Option<String>,
        elem_name: Option<String>,
    ) -> Result<AlsaCard> {
        let card = {
            match card_name {
                Some(name) => audio::get_alsa_card_by_name(name)?,
                None => audio::get_default_alsa_card(),
            }
        };
        let mixer = audio::get_mixer(&card)?;
        let selem_id = audio::get_selem_by_name(
            &mixer,
            elem_name.unwrap_or(String::from("Master")),
        ).unwrap()
            .get_id();

        return Ok(AlsaCard {
            card: card,
            mixer: mixer,
            selem_id: selem_id,
        });
    }

    pub fn selem(&self) -> Selem {
        return audio::get_selems(&self.mixer)
            .nth(self.selem_id.get_index() as usize)
            .unwrap();
    }

    pub fn vol(&self) -> Result<f64> {
        return audio::get_vol(&self.selem());
    }

    pub fn has_mute(&self) -> bool {
        return audio::has_mute(&self.selem());
    }

    pub fn get_mute(&self) -> Result<bool> {
        return audio::get_mute(&self.selem());
    }

    pub fn set_mute(&self, mute: bool) -> Result<()> {
        return audio::set_mute(&self.selem(), mute);
    }
}
