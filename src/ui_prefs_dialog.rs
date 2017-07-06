use app_state::*;
use audio::AudioUser;
use gtk::prelude::*;
use gtk;
use std::rc::Rc;
use support_alsa::*;
use gtk::ResponseType;



// TODO: misbehavior when popup_window is open



pub fn show_prefs_dialog(appstate: Rc<AppS>) {
    if appstate.gui
           .prefs_dialog
           .borrow()
           .is_some() {
        return;
    }

    *appstate.gui.prefs_dialog.borrow_mut() = Some(PrefsDialog::new());
    init_prefs_dialog(&appstate);
    {
        let m_pd = appstate.gui.prefs_dialog.borrow();
        let prefs_dialog = &m_pd.as_ref().unwrap();
        let prefs_dialog_w = &prefs_dialog.prefs_dialog;

        prefs_dialog.from_prefs(&appstate.prefs.borrow());

        prefs_dialog_w.set_transient_for(&appstate.gui.popup_menu.menu_window);
        // TODO: destruct PrefsDialog when clicking Ok/Apply
        prefs_dialog_w.present();
    }
}


/* TODO: do the references get dropped when the dialog window is gone? */
pub fn init_prefs_dialog(appstate: &Rc<AppS>) {

    /* prefs_dialog.connect_show */
    {
        let apps = appstate.clone();
        let m_pd = appstate.gui.prefs_dialog.borrow();
        let pd = m_pd.as_ref().unwrap();
        pd.prefs_dialog.connect_show(move |_| { on_prefs_dialog_show(&apps); });
    }

    /* prefs_dialog.connect_show */
    {
        let apps = appstate.clone();
        let m_pd = appstate.gui.prefs_dialog.borrow();
        let pd = m_pd.as_ref().unwrap();
        pd.prefs_dialog.connect_response(move |_, response_id| {

            let foo = 1;
            if response_id == ResponseType::Ok.into() ||
               response_id == ResponseType::Apply.into() {
                let mut prefs = apps.prefs.borrow_mut();
                let prefs_dialog = apps.gui.prefs_dialog.borrow();
                *prefs = prefs_dialog.as_ref().unwrap().to_prefs();

            }

            if response_id != ResponseType::Apply.into() {
                let mut prefs_dialog = apps.gui.prefs_dialog.borrow_mut();
                prefs_dialog.as_ref().unwrap().prefs_dialog.destroy();
                *prefs_dialog = None;
            }

            if response_id == ResponseType::Ok.into() ||
               response_id == ResponseType::Apply.into() {
                // TODO: update popup, tray_icon, hotkeys, notification and audio
                let prefs = apps.prefs.borrow_mut();
                try_w!(prefs.store_config());
               }

        });
    }

    /*  DEVICE TAB */

    /* card_combo.connect_changed */
    {
        let apps = appstate.clone();
        let m_cc = appstate.gui.prefs_dialog.borrow();
        let card_combo = &m_cc.as_ref().unwrap().card_combo;

        // TODO: refill channel combo
        card_combo.connect_changed(move |_| { on_card_combo_changed(&apps); });
    }
    /* card_combo.connect_changed */
    {
        let apps = appstate.clone();
        let m_cc = appstate.gui.prefs_dialog.borrow();
        let chan_combo = &m_cc.as_ref().unwrap().chan_combo;

        chan_combo.connect_changed(move |_| { on_chan_combo_changed(&apps); });
    }
}


fn on_prefs_dialog_show(appstate: &AppS) {
    let m_cc = appstate.gui.prefs_dialog.borrow();
    let card_combo = &m_cc.as_ref().unwrap().card_combo;
    let m_cc = appstate.gui.prefs_dialog.borrow();
    let chan_combo = &m_cc.as_ref().unwrap().chan_combo;
    let acard = appstate.audio.acard.borrow();


    /* set card combo */
    let cur_card_name = try_w!(acard.card_name(),
                               "Can't get current card name!");
    let available_card_names = get_alsa_card_names();

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
    let available_chan_names = get_selem_names(&acard.mixer);

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
    let m_cc = appstate.gui.prefs_dialog.borrow();
    let card_combo = &m_cc.as_ref().unwrap().card_combo;
    let m_cc = appstate.gui.prefs_dialog.borrow();
    let chan_combo = &m_cc.as_ref().unwrap().chan_combo;
    let active_card_item =
        try_w!(card_combo.get_active_text().ok_or("No active Card item found"));
    let active_chan_item = chan_combo.get_active_id();
    let cur_card_name = {
        let acard = appstate.audio.acard.borrow();
        try_w!(acard.card_name(), "Can't get current card name!")
    };

    if active_card_item != cur_card_name {
        appstate.audio.switch_acard(Some(cur_card_name),
                                    active_chan_item,
                                    AudioUser::PrefsWindow);
    }
}


fn on_chan_combo_changed(appstate: &AppS) {
    let m_cc = appstate.gui.prefs_dialog.borrow();
    let card_combo = &m_cc.as_ref().unwrap().card_combo;
    let m_cc = appstate.gui.prefs_dialog.borrow();
    let chan_combo = &m_cc.as_ref().unwrap().chan_combo;
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
        appstate.audio.switch_acard(cur_card_name,
                                    Some(active_chan_item),
                                    AudioUser::PrefsWindow);
    }
}


fn prefs_dialog_to_prefs(prefs_dialog: &PrefsDialog) {}
