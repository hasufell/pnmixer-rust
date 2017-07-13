//! Helper functions for invoking system commands.


use errors::*;
use glib;
use prefs::Prefs;
use std::error::Error;
use std;



/// Execute an available volume control command asynchronously, starting with
/// the preferences and using some fallback values. If none of these
/// are valid executables in `$PATH`, then return `Err(err)`.
pub fn execute_vol_control_command(prefs: &Prefs) -> Result<()> {
    let m_cmd = prefs.get_avail_vol_control_cmd();

    match m_cmd {
        Some(ref cmd) => execute_command(cmd.as_str()),
        None => bail!("No command found"),
    }
}


/// Try to execute the given command asynchronously via gtk.
pub fn execute_command(cmd: &str) -> Result<()> {
    return glib::spawn_command_line_async(cmd)
               .map_err(|e| {
                            std::io::Error::new(std::io::ErrorKind::Other,
                                                e.description())
                        })
               .from_err();
}
