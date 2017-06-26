extern crate gtk;
extern crate gdk;

use gtk::prelude::*;

pub fn set_slider(vol_scale_adj: & gtk::Adjustment, scale: f64) {
    vol_scale_adj.set_value(scale);
}

