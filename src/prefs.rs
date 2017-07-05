use errors::*;
use std::path::Path;
use glib;
use toml;



const DEFAULT_PREFS: &str = "[device_prefs]
card = \"default\"
channel = \"Master\"

[view_prefs]
draw_vol_meter = true
vol_meter_offset = 10
vol_meter_color = { red = 245, blue = 121, green = 0 }
system_theme = true

[behavior_prefs]
vol_control_cmd = \"\"
vol_scroll_step = 5.0
middle_click_action = \"ToggleMute\"

[notify_prefs]
enable_notifications = true
notifcation_timeout = 1500
notify_mouse_scroll = true
notify_popup = true
notify_external = true";


const VOL_CONTROL_COMMANDS: [&str; 3] = [
    "gnome-alsamixer",
    "xfce4-mixer",
    "alsamixergui"
];



#[derive(Deserialize, Debug, Serialize)]
pub enum MiddleClickAction {
    ToggleMute,
    ShowPreferences,
    VolumeControl,
    CustomCommand(String),
}



#[derive(Deserialize, Debug, Serialize)]
pub struct Prefs {
    pub device_prefs: DevicePrefs,
    pub view_prefs: ViewPrefs,
    pub behavior_prefs: BehaviorPrefs,
    pub notify_prefs: NotifyPrefs,
    // TODO: HotKeys?
}

#[derive(Deserialize, Debug, Serialize)]
pub struct DevicePrefs {
    pub card: String,
    pub channel: String,
    // TODO: normalize volume?
}

#[derive(Deserialize, Debug, Serialize)]
pub struct ViewPrefs {
    pub draw_vol_meter: bool,
    pub vol_meter_offset: i64,
    pub vol_meter_color: VolColor,
    pub system_theme: bool,
    // TODO: Display text folume/text volume pos?
}

#[derive(Deserialize, Debug, Serialize)]
pub struct VolColor {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

#[derive(Deserialize, Debug, Serialize)]
pub struct BehaviorPrefs {
    pub vol_control_cmd: String,
    pub vol_scroll_step: f64,
    pub middle_click_action: MiddleClickAction,
    // TODO: fine scroll step?
}

#[derive(Deserialize, Debug, Serialize)]
pub struct NotifyPrefs {
    pub enable_notifications: bool,
    pub notifcation_timeout: i64,
    pub notify_mouse_scroll: bool,
    pub notify_popup: bool,
    pub notify_external: bool,
    // TODO: notify_hotkeys?
}


impl Prefs {
    // pub fn set_blah(&mut self) {
        // self.vol_scroll_step = 5.0;
    // }

    // pub fn new() -> Prefs {
        // // load from config

    // }

    // pub fn reload_from_config(&self) {

    // }


    // pub fn save_to_config() -> Result<()> {

    // }


    // fn config_path() -> String {

    // }


    fn ensure_config_path() {

    }

    pub fn new_from_def() -> Prefs {
        let prefs: Prefs = toml::from_str(DEFAULT_PREFS).unwrap();
        return prefs;
    }


    pub fn to_str(&self) -> String {
        return toml::to_string(self).unwrap();
    }
}

