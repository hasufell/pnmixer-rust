use audio::{Audio, AudioUser};
use errors::*;
use prefs::*;


#[derive(Clone, Copy, Debug)]
pub enum VolDir {
    Up,
    Down,
    Unknown,
}


pub fn vol_change_to_voldir(old: f64, new: f64) -> VolDir {
    if old < new {
        return VolDir::Up;
    } else if old > new {
        return VolDir::Down;
    } else {
        return VolDir::Unknown;
    }
}


pub fn lrint(v: f64, dir: VolDir) -> f64 {
    match dir {
        VolDir::Up => v.ceil(),
        VolDir::Down => v.floor(),
        _ => v,
    }
}

pub fn audio_reload(audio: &Audio,
                    prefs: &Prefs,
                    user: AudioUser)
                    -> Result<()> {
    let card = &prefs.device_prefs.card;
    let channel = &prefs.device_prefs.channel;
    // TODO: is this clone safe?
    return audio.switch_acard(Some(card.clone()), Some(channel.clone()), user);
}


pub fn vol_to_percent(vol: i64, range: (i64, i64)) -> Result<f64> {
    let (min, max) = range;
    ensure!(min < max, "Invalid playback volume range [{} - {}]", min, max);
    let perc = ((vol - min) as f64) / ((max - min) as f64) * 100.0;
    return Ok(perc);
}


pub fn percent_to_vol(vol: f64, range: (i64, i64), dir: VolDir) -> Result<i64> {
    let (min, max) = range;
    ensure!(min < max, "Invalid playback volume range [{} - {}]", min, max);

    let _v = lrint(vol / 100.0 * ((max - min) as f64), dir) + (min as f64);
    return Ok(_v as i64);
}
