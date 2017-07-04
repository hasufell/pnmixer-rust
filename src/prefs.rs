use errors::*;
use std::path::Path;
use glib;
use glib::KeyFile;



const DEFAULT_PREFS: &str = "[PNMixer]\n
AlsaCard=(default)\n
AlsaChannel=Master\n
DrawVolMeter=True\n
VolMeterOffset=10\n
VolMeterColor=245;121;0;\n
SystemTheme=true\n
VolControlCommand=xfce4-mixer\n
VolControlStep=5\n
MiddleClickAction=0\n
CustomCommand=\n
EnableNotifications=true\n
NotificationTimeout=1500\n
MouseNotifications=true\n
PopupNotifcations=true\n
ExternalNotifications=true\n";


const VOL_CONTROL_COMMANDS: [&str; 3] = [
    "gnome-alsamixer",
    "xfce4-mixer",
    "alsamixergui"
];



pub enum MiddleClickAction {
    ToggleMute,
    ShowPreferences,
    VolumeControl,
    CustomCommand(String),
}


struct Prefs {
    key_file: glib::KeyFile,

    /* device prefs */
    pub card: String,
    pub channel: String,
    // TODO: normalize volume?

    /* view prefs */
    pub draw_vol_meter: bool,
    pub vol_meter_offset: i64,
    pub vol_meter_color: (u8, u8, u8),
    pub system_theme: bool,
    // TODO: Display text folume/text volume pos?

    /* behavior */
    pub vol_control_cmd: String,
    pub vol_scroll_step: f64,
    pub middle_click_action: MiddleClickAction,
    // TODO: fine scroll step?

    // TODO: HotKeys?

    /* notifications */
    pub enable_notifications: bool,
    pub notifcation_timeout: i64,
    pub notify_mouse_scroll: bool,
    pub notify_popup: bool,
    pub notify_external: bool,
    // TODO: notify_hotkeys?
}


impl Prefs {
    pub fn new() -> Prefs {
        // load from config

    }

    pub fn reload_from_config(&self) {

    }


    pub fn save_to_config() -> Result<()> {

    }


    fn config_path() -> String {

    }


    fn ensure_config_path() {

    }
}

