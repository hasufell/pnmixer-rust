use gtk;



pub struct AppS {
    /* we keep this to ensure the lifetime is across the whole application */
    pub status_icon: gtk::StatusIcon,

    pub builder_popup: gtk::Builder,
}

