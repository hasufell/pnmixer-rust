use audio::{Audio, AudioUser};
use errors::*;
use prefs::*;



pub fn audio_reload(audio: &Audio,
                    prefs: &Prefs,
                    user: AudioUser)
                    -> Result<()> {
    let card = &prefs.device_prefs.card;
    let channel = &prefs.device_prefs.channel;
    // TODO: is this clone safe?
    return audio.switch_acard(Some(card.clone()), Some(channel.clone()), user);
}
