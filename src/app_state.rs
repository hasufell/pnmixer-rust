use gtk;

use alsa::poll::PollDescriptors;
use alsa::card::Card;
use alsa::mixer::{Mixer, SelemId, Selem};
use audio;
use errors::*;
use std::rc::Rc;
use std::cell::RefCell;


// TODO: fix import
use libc::pollfd;

pub struct AppS {
    /* we keep this to ensure the lifetime is across the whole application */
    pub status_icon: gtk::StatusIcon,

    pub builder_popup: gtk::Builder,
}

// TODO: implement free/destructor
pub struct AlsaCard {
    _cannot_construct: (),
    pub card: Card,
    pub mixer: Mixer,
    pub selem_id: SelemId,
    pub watch_ids: Vec<u32>,
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
        let rc_mixer = RefCell::new(audio::get_mixer(&card)?);
        let selem_id = audio::get_selem_by_name(
            &mixer,
            elem_name.unwrap_or(String::from("Master")),
        ).unwrap()
            .get_id();
        let vec_pollfd = PollDescriptors::get(&mixer)?;
        // let watch_ids = vec![];
        let watch_ids = audio::watch_poll_descriptors(vec_pollfd, rc_mixer);

        return Ok(AlsaCard {
            _cannot_construct: (),
            card: card,
            mixer: mixer,
            selem_id: selem_id,
            watch_ids: watch_ids,
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

    pub fn set_vol(&self, new_vol: f64) -> Result<()> {
        return audio::set_vol(&self.selem(), new_vol);
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
