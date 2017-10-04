//! Pulseaudio backend subsystem.

use audio::frontend::*;
use errors::*;
use libc;
use libpulse_sys::*;
use std::cell::RefCell;
use std::ffi::{CString, CStr};
use std::mem;
use std::os::raw::c_char;
use std::ptr;
use support::pulseaudio::*;
use support::audio::*;


// TODO: get info based on index, not descr.
//
// TODO: how to hook pulseaudio events? port change?
// TODO: how to handle channels


// TODO: update when sink changes, only name and description are const
#[derive(Clone, Debug)]
pub struct Sink {
    pub name: String,
    pub index: u32,
    pub description: String,
    pub channels: u8,
}


pub struct PABackend {
    _cannot_construct: (),
    m: *mut pa_threaded_mainloop,
    c: *mut pa_context,
    pub sink: RefCell<Sink>,
}


impl PABackend {
    pub fn new(sink_desc: Option<String>) -> Result<Self> {
        unsafe {
            let mainloop: *mut pa_threaded_mainloop = pa_threaded_mainloop_new();

            ensure!(!mainloop.is_null(), "Main loop is null");

            let api: *mut pa_mainloop_api = pa_threaded_mainloop_get_api(mainloop);

            let context_name = CString::new("pnmixer-rs").unwrap();
            let context: *mut pa_context = pa_context_new(api,
                                                          context_name.as_ptr());

            if context.is_null() {
                pa_threaded_mainloop_free(mainloop);
                bail!("Couldn't create context");
            }

            pa_context_set_state_callback(context,
                                          Some(context_state_cb),
                                          mainloop as *mut libc::c_void);
            // TODO: don't spawn new daemon
            let cret = pa_context_connect(context,
                                         ptr::null_mut(),
                                         0,
                                         ptr::null_mut());

            if cret < 0 {
                pa_context_unref(context);
                pa_threaded_mainloop_free(mainloop);
                bail!("Couldn't connect to daemon");
            }

            let mret = pa_threaded_mainloop_start(mainloop);

            if mret < 0 {
                pa_context_unref(context);
                pa_threaded_mainloop_free(mainloop);
                bail!("Couldn't start main loop");
            }

            pa_threaded_mainloop_lock(mainloop);
            while !CONTEXT_READY {
                pa_threaded_mainloop_wait(mainloop);
            }
            pa_threaded_mainloop_accept(mainloop);
            pa_threaded_mainloop_unlock(mainloop);
            CONTEXT_READY = false;

            let sink = {
                match sink_desc.as_ref().map(|s| s.as_str()) {
                    Some("(default)") => get_first_sink(mainloop, context)?,
                    Some(sd) => {
                        let mysink = get_sink_by_desc(mainloop, context, sd);
                        match mysink {
                            Ok(s) => s,
                            Err(_) => {
                                warn!("Could not find sink with name {}, trying others", sd);
                                get_first_sink(mainloop, context)?
                            }
                        }

                    }
                    None => get_first_sink(mainloop, context)?
                }

            };

            return Ok(PABackend {
                _cannot_construct: (),
                m: mainloop,
                c: context,
                sink: RefCell::new(sink),
            })
        }
    }

    pub fn get_vol(&self) -> Result<f64> {

        let mut vol: u32 = 0;
        unsafe {

            pa_threaded_mainloop_lock(self.m);
            let data = &mut(self, &mut vol);
            let sink_name = CString::new(self.sink.borrow().name.clone()).unwrap().into_raw();
            let o = pa_context_get_sink_info_by_name(self.c,
                                                     sink_name,
                                                     Some(get_sink_vol),
                                                     data as *mut _ as *mut libc::c_void);
            if o.is_null() {
                pa_threaded_mainloop_unlock(self.m);
                bail!("Failed to initialize PA operation!");
            }

            while pa_operation_get_state(o) == PA_OPERATION_RUNNING {
                pa_threaded_mainloop_wait(self.m);
            }
            pa_operation_unref(o);
            pa_threaded_mainloop_unlock(self.m);

            let _ = CString::from_raw(sink_name);

        }
        unsafe {
            return Ok(pa_sw_volume_to_linear(vol) * 100.0);
        }
    }

    pub fn set_vol(&self, new_vol: f64, dir: VolDir) -> Result<()> {
        let mut res: Result<()> = Err("No value".into());
        unsafe {
            pa_threaded_mainloop_lock(self.m);
            let data = &mut(self, &mut res);
            let sink_name = CString::new(self.sink.borrow().name.clone()).unwrap().into_raw();

            let new_vol = pa_sw_volume_from_linear(new_vol / 100.0);
            let mut vol_arr: [u32; 32] = [0; 32];
            for c in 0..(self.sink.borrow().channels) {
                vol_arr[c as usize] = new_vol as u32;
            }
            let mut new_cvol = Struct_pa_cvolume {
                channels: self.sink.borrow().channels,
                values: vol_arr,
            };

            assert!(pa_cvolume_valid(&mut new_cvol as *mut pa_cvolume) != 0, "Invalid cvolume");

            let o = pa_context_set_sink_volume_by_name(self.c,
                                                       sink_name,
                                                       &new_cvol as *const pa_cvolume,
                                                       Some(set_sink_vol),
                                                       data as *mut _ as *mut libc::c_void);

            if o.is_null() {
                pa_threaded_mainloop_unlock(self.m);
                bail!("Failed to initialize PA operation!");
            }

            while pa_operation_get_state(o) == PA_OPERATION_RUNNING {
                pa_threaded_mainloop_wait(self.m);
            }
            pa_operation_unref(o);
            pa_threaded_mainloop_unlock(self.m);
        }

        return res;
    }

    pub fn has_mute(&self) -> bool {
        return true;
    }

    pub fn get_mute(&self) -> Result<bool> {
        let mut mute: bool = false;
        unsafe {

            pa_threaded_mainloop_lock(self.m);
            let data = &mut(self, &mut mute);
            let sink_name = CString::new(self.sink.borrow().name.clone()).unwrap().into_raw();
            let o = pa_context_get_sink_info_by_name(self.c,
                                                     sink_name,
                                                     Some(get_sink_mute),
                                                     data as *mut _ as *mut libc::c_void);
            if o.is_null() {
                pa_threaded_mainloop_unlock(self.m);
                bail!("Failed to initialize PA operation!");
            }

            while pa_operation_get_state(o) == PA_OPERATION_RUNNING {
                pa_threaded_mainloop_wait(self.m);
            }
            pa_operation_unref(o);
            pa_threaded_mainloop_unlock(self.m);

            let _ = CString::from_raw(sink_name);
        }
        return Ok(mute);
    }

    pub fn set_mute(&self, mute: bool) -> Result<()> {
        let mut res: Result<()> = Err("No value".into());
        unsafe {
            pa_threaded_mainloop_lock(self.m);
            let data = &mut(self, &mut res);
            let sink_name = CString::new(self.sink.borrow().name.clone()).unwrap().into_raw();

            let o = pa_context_set_sink_mute_by_name(self.c,
                                                     sink_name,
                                                     mute as i32,
                                                     Some(set_sink_mute),
                                                     data as *mut _ as *mut libc::c_void);

            if o.is_null() {
                pa_threaded_mainloop_unlock(self.m);
                bail!("Failed to initialize PA operation!");
            }

            while pa_operation_get_state(o) == PA_OPERATION_RUNNING {
                pa_threaded_mainloop_wait(self.m);
            }
            pa_operation_unref(o);
            pa_threaded_mainloop_unlock(self.m);
        }

        return res;
    }
}


impl Drop for PABackend {
    fn drop(&mut self) {
        unsafe {
            debug!("Stopping PA main loop");
            pa_threaded_mainloop_stop(self.m);
            debug!("Freeing PA context");
            pa_context_unref(self.c);
            debug!("Freeing main loop");
            pa_threaded_mainloop_free(self.m);
        }
    }
}





static mut CONTEXT_READY: bool = false;

// TODO: proper error handling
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


// TODO: Better error handling.
unsafe extern "C" fn get_sink_vol(
        ctx: *mut pa_context,
        i: *const pa_sink_info,
        eol: i32,
        data: *mut libc::c_void) {
    let (_self, res) = *(data as *mut (*mut PABackend,
                                       *mut u32));
    assert!(!(*_self).m.is_null(), "Mainloop is null");

    if i.is_null() {
        return
    }

    *res = (*i).volume.values[0];

    pa_threaded_mainloop_signal((*_self).m, 0);
}

// TODO: Better error handling.
unsafe extern "C" fn get_sink_mute(
        ctx: *mut pa_context,
        i: *const pa_sink_info,
        eol: i32,
        data: *mut libc::c_void) {
    let (_self, res) = *(data as *mut (*mut PABackend,
                                       *mut bool));
    assert!(!(*_self).m.is_null(), "Mainloop is null");

    if i.is_null() {
        return
    }

    *res = (*i).mute != 0;

    pa_threaded_mainloop_signal((*_self).m, 0);
}

// TODO: Missing error handling.
unsafe extern "C" fn set_sink_vol(
        ctx: *mut pa_context,
        success: i32,
        data: *mut libc::c_void) {
    let (_self, res) = *(data as *mut (*mut PABackend,
                                       *mut Result<()>));
    assert!(!(*_self).m.is_null(), "Mainloop is null");

    if success > 0 {
        *res = Ok(());
    } else {
        *res = Err("Failed to set volume".into());
    }

    pa_threaded_mainloop_signal((*_self).m, 0);
}


// TODO: Missing error handling.
// TODO: same as 'set_sink_vol'
unsafe extern "C" fn set_sink_mute(
        ctx: *mut pa_context,
        success: i32,
        data: *mut libc::c_void) {
    let (_self, res) = *(data as *mut (*mut PABackend,
                                       *mut Result<()>));
    assert!(!(*_self).m.is_null(), "Mainloop is null");

    if success > 0 {
        *res = Ok(());
    } else {
        *res = Err("Failed to set volume".into());
    }

    pa_threaded_mainloop_signal((*_self).m, 0);
}
