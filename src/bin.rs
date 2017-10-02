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


static mut CONTEXT_READY: bool = false;


unsafe extern "C" fn context_state_cb(
        ctx: *mut pa_context, data: *mut libc::c_void) {

    let mainloop: *mut pa_threaded_mainloop = data as *mut pa_threaded_mainloop;
    let state = pa_context_get_state(ctx);

    match state {
        PA_CONTEXT_READY => {
            CONTEXT_READY = true;
            pa_threaded_mainloop_signal(mainloop, 1);
        },
        _ => ()
    }

}

fn main() {
    unsafe {
        let mainloop: *mut pa_threaded_mainloop = pa_threaded_mainloop_new();

        if mainloop.is_null() {
            panic!("Oh no");
        }

        let api: *mut pa_mainloop_api = pa_threaded_mainloop_get_api(mainloop);

        let context_name = CString::new("pnmixer-rs").unwrap();
        let context: *mut pa_context = pa_context_new(api,
                                                      context_name.as_ptr());

        if context.is_null() {
            panic!("Oh no");
        }

        pa_context_set_state_callback(context,
                                      Some(context_state_cb),
                                      mainloop as *mut libc::c_void);
        let ret = pa_context_connect(context,
                                     std::ptr::null_mut(),
                                     0,
                                     std::ptr::null_mut());

        if ret < 0 {
            panic!("Oh no");
        }

        let ret = pa_threaded_mainloop_start(mainloop);

        if ret < 0 {
            panic!("Oh no");
        }

        pa_threaded_mainloop_lock(mainloop);
        while !CONTEXT_READY {
            pa_threaded_mainloop_wait(mainloop);
        }
        pa_threaded_mainloop_accept(mainloop);
        pa_threaded_mainloop_unlock(mainloop);
        CONTEXT_READY = false;

    }


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
