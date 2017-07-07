use errors::*;
use glib;
use prefs::Prefs;
use std::error::Error;
use std;



pub fn execute_vol_control_command(prefs: &Prefs) -> Result<()> {
    let m_cmd = prefs.get_avail_vol_control_cmd();

    match m_cmd {
        Some(ref cmd) => execute_command(cmd.as_str()),
        None => bail!("No command found"),
    }
}


pub fn execute_command(cmd: &str) -> Result<()> {
    return glib::spawn_command_line_async(cmd)
               .map_err(|e| {
                            std::io::Error::new(std::io::ErrorKind::Other,
                                                e.description())
                        })
               .from_err();
}
