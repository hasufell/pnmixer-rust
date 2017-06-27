use gtk;

use alsa::card::Card;
use alsa::mixer::{Mixer, Selem};
use std::cell::Cell;

pub struct AppS {
    /* we keep this to ensure the lifetime is across the whole application */
    pub status_icon: gtk::StatusIcon,

    pub builder_popup: gtk::Builder,
}

pub struct AlsaCard<'a> {
    pub card: Cell<Card>,
    pub mixer: Cell<Mixer>,
    pub selem: Cell<Selem<'a>>,
}

