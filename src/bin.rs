#![feature(alloc_system)]
extern crate alloc_system;
extern crate getopts;
extern crate libpulse_sys;
extern crate libc;

extern crate pnmixerlib;

use pnmixerlib::*;

use app_state::*;
use getopts::Options;
use std::rc::Rc;
use std::env;
use audio::pulseaudio;
use libpulse_sys::*;
use std::ffi::{CString, CStr};
use std::os::raw::c_char;
use std::ptr;
use audio::pulseaudio::*;
use support::audio::VolDir;



fn main() {




    let args: Vec<String> = env::args().collect();

    let mut opts = Options::new();
    opts.optflag("h", "help", "Show help");
    opts.optopt("", "log-to-file",
                "Log files to the specified dir instead of stderr",
                "DIRECTORY");
    opts.optflagopt("l", "log-level",
                "Set the log level (trace/debug/info/warn/error/off)",
                "LEVEL");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => { m }
        Err(f) => { panic!(f.to_string()) }
    };

    if matches.opt_present("h") {
        print_usage(opts);
        return;
    }

    let log_dir = matches.opt_str("log-to-file");
    let log_level = matches.opt_default("log-level", "debug").map(|s| {
        match s.to_lowercase().as_str() {
            "trace" => flexi_logger::LogLevelFilter::Trace,
            "debug" => flexi_logger::LogLevelFilter::Debug,
            "info"  => flexi_logger::LogLevelFilter::Info,
            "warn"  => flexi_logger::LogLevelFilter::Warn,
            "error" => flexi_logger::LogLevelFilter::Error,
            "off"   => flexi_logger::LogLevelFilter::Off,
            _       => flexi_logger::LogLevelFilter::Debug,
        }
    }).unwrap_or(flexi_logger::LogLevelFilter::Off);

    let mut flogger = flexi_logger::Logger::with(
        flexi_logger::LogSpecification::default(log_level).build());

    if let Some(dir) = log_dir {
        flogger = flogger.log_to_file().directory(dir);
    }

    flogger
        .start()
        .unwrap_or_else(|e|{panic!("Logger initialization failed with {}",e)});


    let pa = PABackend::new(Some(String::from("Built-in Audio Analog Stereo"))).unwrap();
    println!("Sink: {:?}", pa.sink);
    println!("Volume before: {:?}", pa.get_vol());
    pa.set_vol(80.0, VolDir::Up);
    println!("Volume after: {:?}", pa.get_vol());
    println!("Mute before: {:?}", pa.get_mute());
    println!("PA_VOLUME_NORM: {:?}", PA_VOLUME_NORM);
    // pa.set_mute(true);
    // println!("Mute after: {:?}", pa.get_mute());

    return;

    gtk::init()
        .unwrap_or_else(|e| panic!("Gtk initialization failed with {}", e));

    let apps = Rc::new(new_alsa_appstate());

    ui::entry::init(apps);

    gtk::main();

    // TODO: clean deallocation?
}


fn print_usage(opts: Options) {
    let brief = format!("Usage: pnmixer-rs [options]");
    print!("{}", opts.usage(&brief));
}
