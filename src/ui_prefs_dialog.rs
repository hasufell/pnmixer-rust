use app_state::*;
use gtk::prelude::*;
use std::rc::Rc;
use gtk;
use alsa_pn;


pub fn init_prefs_dialog(appstate: Rc<AppS>) {
    /* prefs_dialog.connect_show */
    {
        let apps = appstate.clone();
        let prefs_dialog = &appstate.clone()
                                .gui
                                .prefs_dialog
                                .prefs_dialog;
        prefs_dialog.connect_show(move |_| { on_prefs_dialog_show(&apps); });
    }

    /* card_combo.connect_changed */
    {
        let apps = appstate.clone();
        let card_combo = &appstate.gui.prefs_dialog.card_combo;

        card_combo.connect_changed(move |_| { on_card_combo_changed(&apps); });
    }
    /* card_combo.connect_changed */
    {
        let apps = appstate.clone();
        let chan_combo = &appstate.gui.prefs_dialog.chan_combo;

        chan_combo.connect_changed(move |_| { on_chan_combo_changed(&apps); });
    }
}


fn on_prefs_dialog_show(appstate: &AppS) {
    let card_combo = &appstate.gui.prefs_dialog.card_combo;
    let chan_combo = &appstate.gui.prefs_dialog.chan_combo;
    let acard = appstate.audio.acard.borrow();


    /* set card combo */
    let cur_card_name = try_w!(acard.card_name(),
                               "Can't get current card name!");
    let available_card_names = alsa_pn::get_alsa_card_names();

    /* set_active_id doesn't work, so save the index */
    let mut c_index: i32 = -1;
    for i in 0..available_card_names.len() {
        let name = available_card_names.get(i).unwrap();
        if *name == cur_card_name {
            c_index = i as i32;
        }
        card_combo.append_text(&name);
    }

    // TODO, block signal?
    card_combo.set_active(c_index);



    /* set chan combo */
    let cur_chan_name = try_w!(acard.chan_name());
    let available_chan_names = alsa_pn::get_selem_names(&acard.mixer);

    /* set_active_id doesn't work, so save the index */
    let mut c_index: i32 = -1;
    for i in 0..available_chan_names.len() {
        let name = available_chan_names.get(i).unwrap();
        if *name == cur_chan_name {
            c_index = i as i32;
        }
        chan_combo.append_text(&name);
    }

    /* TODO, block signal?`*/
    chan_combo.set_active(c_index);

}


fn on_card_combo_changed(appstate: &AppS) {
    let card_combo = &appstate.gui.prefs_dialog.card_combo;
    let chan_combo = &appstate.gui.prefs_dialog.chan_combo;
    let active_card_item =
        try_w!(card_combo.get_active_text().ok_or("No active Card item found"));
    let active_chan_item =
        chan_combo.get_active_id();
    let cur_card_name = {
        let acard = appstate.audio.acard.borrow();
        try_w!(acard.card_name(),
                               "Can't get current card name!")
    };

    if active_card_item != cur_card_name {
        appstate.audio.switch_acard(Some(cur_card_name), active_chan_item);
    }
}


fn on_chan_combo_changed(appstate: &AppS) {
    let card_combo = &appstate.gui.prefs_dialog.card_combo;
    let chan_combo = &appstate.gui.prefs_dialog.chan_combo;
    let active_chan_item =
        try_w!(chan_combo.get_active_text().ok_or("No active Chan item found"));
    let cur_card_name = {
        let acard = appstate.audio.acard.borrow();
        acard.card_name().ok()
    };
    let cur_chan_name = {
        let acard = appstate.audio.acard.borrow();
        try_w!(acard.chan_name())
    };

    if active_chan_item != cur_chan_name {
        appstate.audio.switch_acard(cur_card_name, Some(active_chan_item));
    }
}

