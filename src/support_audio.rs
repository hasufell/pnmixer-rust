#![allow(missing_docs)] // enum

//! Helper functions of the audio subsystem.
//!
//! These functions are not directly connected to the `Audio` struct,
//! but are important helpers.


use audio::{Audio, AudioUser};
use errors::*;
use prefs::*;


#[derive(Clone, Copy, Debug)]
/// The direction of a volume change.
pub enum VolDir {
    Up,
    Down,
    Unknown,
}


/// Convert a volume change to the `VolDir` type.
/// ## `old`
/// The old volume value.
/// ## `new`
/// The new volume value.
///
/// # Returns
///
/// The direction of the volume change as `Voldir`.
pub fn vol_change_to_voldir(old: f64, new: f64) -> VolDir {
    if old < new {
        return VolDir::Up;
    } else if old > new {
        return VolDir::Down;
    } else {
        return VolDir::Unknown;
    }
}


/// Kinda mimics `lrint` from libm. If the direction of the volume change
/// is `Up` then calls `ceil()`, if it's `Down`, then calls `floor()`, otherwise
/// returns the value unchanged.
pub fn lrint(v: f64, dir: VolDir) -> f64 {
    match dir {
        VolDir::Up => v.ceil(),
        VolDir::Down => v.floor(),
        _ => v,
    }
}


/// Reload the audio system.
pub fn audio_reload(audio: &Audio,
                    prefs: &Prefs,
                    user: AudioUser)
                    -> Result<()> {
    let card = &prefs.device_prefs.card;
    let channel = &prefs.device_prefs.channel;
    // TODO: is this clone safe?
    return audio.switch_acard(Some(card.clone()), Some(channel.clone()), user);
}


/// Converts the actual volume of the audio configuration, which depends
/// on the volume range, to a scale of 0-100, reprenting the percentage
/// of the volume level.
pub fn vol_to_percent(vol: i64, range: (i64, i64)) -> Result<f64> {
    let (min, max) = range;
    ensure!(min < max,
            "Invalid playback volume range [{} - {}]",
            min,
            max);
    let perc = ((vol - min) as f64) / ((max - min) as f64) * 100.0;
    return Ok(perc);
}


/// Converts the percentage of the volume level (0-100) back to the actual
/// low-level representation of the volume, which depends on the volume
/// range.
pub fn percent_to_vol(vol: f64, range: (i64, i64), dir: VolDir) -> Result<i64> {
    let (min, max) = range;
    ensure!(min < max,
            "Invalid playback volume range [{} - {}]",
            min,
            max);

    let _v = lrint(vol / 100.0 * ((max - min) as f64), dir) + (min as f64);
    return Ok(_v as i64);
}
