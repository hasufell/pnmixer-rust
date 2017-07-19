#![allow(missing_docs)]

//! The preferences subsystem.
//!
//! These are the global application preferences, which can be set
//! by the user. They read from a file in TOML format, presented
//! in the preferences dialog and saved back to the file on request.


use errors::*;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fs::File;
use std::io::prelude::*;
use std;
use toml;
use which;
use xdg;




const VOL_CONTROL_COMMANDS: [&str; 3] =
    ["gnome-alsamixer", "xfce4-mixer", "alsamixergui"];



#[derive(Deserialize, Debug, Serialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
/// When the tray icon is middle-clicked.
pub enum MiddleClickAction {
    ToggleMute,
    ShowPreferences,
    VolumeControl,
    CustomCommand,
}

impl Default for MiddleClickAction {
    fn default() -> MiddleClickAction {
        return MiddleClickAction::ToggleMute;
    }
}


impl From<i32> for MiddleClickAction {
    fn from(i: i32) -> Self {
        match i {
            0 => MiddleClickAction::ToggleMute,
            1 => MiddleClickAction::ShowPreferences,
            2 => MiddleClickAction::VolumeControl,
            3 => MiddleClickAction::CustomCommand,
            _ => MiddleClickAction::ToggleMute,
        }
    }
}


impl From<MiddleClickAction> for i32 {
    fn from(action: MiddleClickAction) -> Self {
        match action {
            MiddleClickAction::ToggleMute => 0,
            MiddleClickAction::ShowPreferences => 1,
            MiddleClickAction::VolumeControl => 2,
            MiddleClickAction::CustomCommand => 3,
        }
    }
}



#[derive(Deserialize, Debug, Serialize)]
#[serde(default)]
/// Device preferences tab.
pub struct DevicePrefs {
    pub card: String,
    pub channel: String,
    // TODO: normalize volume?
}

impl Default for DevicePrefs {
    fn default() -> DevicePrefs {
        return DevicePrefs {
            card: String::from("(default)"),
            channel: String::from("Master"),
        };
    }
}


#[derive(Deserialize, Debug, Serialize)]
#[serde(default)]
/// View preferences tab.
pub struct ViewPrefs {
    pub draw_vol_meter: bool,
    pub vol_meter_offset: i32,
    pub system_theme: bool,
    pub vol_meter_color: VolColor,
    // TODO: Display text folume/text volume pos?
}

impl Default for ViewPrefs {
    fn default() -> ViewPrefs {
        return ViewPrefs {
            draw_vol_meter: true,
            vol_meter_offset: 10,
            system_theme: true,
            vol_meter_color: VolColor::default(),
        };
    }
}


#[derive(Deserialize, Debug, Serialize)]
#[serde(default)]
/// Volume color setting in the view preferences tab.
pub struct VolColor {
    pub red: f64,
    pub green: f64,
    pub blue: f64,
}

impl Default for VolColor {
    fn default() -> VolColor {
        return VolColor {
            red: 0.960784313725,
            green: 0.705882352941,
            blue: 0.0,
        };
    }
}


#[derive(Deserialize, Debug, Serialize)]
#[serde(default)]
/// Behavior preferences tab.
pub struct BehaviorPrefs {
    pub unmute_on_vol_change: bool,
    pub vol_control_cmd: Option<String>,
    pub vol_scroll_step: f64,
    pub vol_fine_scroll_step: f64,
    pub middle_click_action: MiddleClickAction,
    pub custom_command: Option<String>, // TODO: fine scroll step?
}

impl Default for BehaviorPrefs {
    fn default() -> BehaviorPrefs {
        return BehaviorPrefs {
            unmute_on_vol_change: true,
            vol_control_cmd: None,
            vol_scroll_step: 5.0,
            vol_fine_scroll_step: 1.0,
            middle_click_action: MiddleClickAction::default(),
            custom_command: None,
        };
    }
}


#[cfg(feature = "notify")]
#[derive(Deserialize, Debug, Serialize)]
#[serde(default)]
/// Notifications preferences tab.
pub struct NotifyPrefs {
    pub enable_notifications: bool,
    pub notifcation_timeout: i64,
    pub notify_mouse_scroll: bool,
    pub notify_popup: bool,
    pub notify_external: bool,
    pub notify_hotkeys: bool,
}

#[cfg(feature = "notify")]
impl Default for NotifyPrefs {
    fn default() -> NotifyPrefs {
        return NotifyPrefs {
            enable_notifications: true,
            notifcation_timeout: 1500,
            notify_mouse_scroll: true,
            notify_popup: true,
            notify_external: true,
            notify_hotkeys: true,
        };
    }
}


#[derive(Deserialize, Debug, Serialize, Default)]
/// Hotkey preferences.
/// The `String`s represent gtk accelerator strings.
pub struct HotkeyPrefs {
    pub enable_hotkeys: bool,
    pub mute_unmute_key: Option<String>,
    pub vol_up_key: Option<String>,
    pub vol_down_key: Option<String>,
}


#[derive(Deserialize, Debug, Serialize, Default)]
#[serde(default)]
/// Main preferences struct, holding all sub-preferences.
pub struct Prefs {
    pub device_prefs: DevicePrefs,
    pub view_prefs: ViewPrefs,
    pub behavior_prefs: BehaviorPrefs,
    #[cfg(feature = "notify")]
    pub notify_prefs: NotifyPrefs,
    pub hotkey_prefs: HotkeyPrefs,
}

impl Prefs {
    pub fn new() -> Result<Prefs> {
        let m_config_file = get_xdg_dirs().find_config_file("pnmixer.toml");
        match m_config_file {
            Some(c) => {
                debug!("Config file present at {:?}, using it.", c);

                let mut f = File::open(c)?;
                let mut buffer = vec![];
                f.read_to_end(&mut buffer)?;

                let prefs = toml::from_slice(buffer.as_slice())?;

                return Ok(prefs);
            }
            None => {
                debug!("No config file present, creating one with defaults.");

                let prefs = Prefs::default();
                prefs.store_config()?;

                return Ok(prefs);
            }
        }

    }


    // TODO: unused
    /// Reload the current preferences from the config file.
    pub fn reload_config(&mut self) -> Result<()> {
        debug!("Reloading config...");

        let new_prefs = Prefs::new()?;
        *self = new_prefs;

        return Ok(());
    }


    /// Store the current preferences to the config file.
    pub fn store_config(&self) -> Result<()> {
        let config_path = get_xdg_dirs()
            .place_config_file("pnmixer.toml")
            .chain_err(|| {
                format!(
                    "Could not create config directory at {:?}",
                    get_xdg_dirs().get_config_home()
                )
            })?;

        debug!("Storing config in {:?}", config_path);

        let mut f = File::create(config_path.clone()).chain_err(|| {
            format!(
                "Could not open/create config file {:?} for writing",
                config_path
            )
        })?;
        f.write_all(self.to_str().as_bytes()).chain_err(|| {
            format!("Could not write to config file {:?}", config_path)
        })?;

        return Ok(());
    }


    /// Conver the current preferences to a viewable String.
    pub fn to_str(&self) -> String {
        return toml::to_string(self).unwrap();
    }


    /// Get an available volume control command, which must exist in `$PATH`.
    /// Tries hard to fine one,
    /// starting with the given preference setting and falling back to the
    /// `VOL_CONTROL_COMMANDS` slice.
    pub fn get_avail_vol_control_cmd(&self) -> Option<String> {
        match self.behavior_prefs.vol_control_cmd {
            Some(ref c) => return Some(c.clone()),
            None => {
                for command in VOL_CONTROL_COMMANDS.iter() {
                    if which::which(command).is_ok() {
                        return Some(String::from(*command));
                    }
                }
            }
        }

        return None;
    }
}

impl Display for Prefs {
    fn fmt(
        &self,
        f: &mut Formatter,
    ) -> std::result::Result<(), std::fmt::Error> {
        let s = self.to_str();
        return write!(f, "{}", s);
    }
}


/// Get the set of XDG directories, relative to our project.
fn get_xdg_dirs() -> xdg::BaseDirectories {
    return xdg::BaseDirectories::with_prefix("pnmixer-rs").unwrap();
}
