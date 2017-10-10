#![allow(missing_docs)]


//! Pulseaudio backend subsystem.

use audio::frontend::*;
use errors::*;
use libc;
use libpulse_sys::*;
use std::cell::Cell;
use std::cell::RefCell;
use std::ffi::{CString};
use std::ptr;
use support::pulseaudio::*;
use support::audio::*;


pub const PA_VOLUME_MUTED: i64 = 0x0;
pub const PA_VOLUME_NORM: i64 = 0x10000;



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

impl Sink {
    pub fn new(sink_desc: Option<String>,
               chan_name: Option<String>,
               mainloop: *mut pa_threaded_mainloop,
               context: *mut pa_context) -> Result<Self> {
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

        return Ok(sink);
    }
}


pub struct PABackend {
    _cannot_construct: (),
    m: *mut pa_threaded_mainloop,
    c: *mut pa_context,
    pub sink: RefCell<Sink>,
    pub scroll_step: Cell<u32>,
    pub handlers: Handlers,
}


impl PABackend {
    pub fn new(sink_desc: Option<String>, chan_name: Option<String>) -> Result<Self> {
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

            let sink = Sink::new(sink_desc, chan_name, mainloop, context)?;



            pa_threaded_mainloop_lock(mainloop);

            let mut success: bool = false;
            let data = &mut(mainloop, &mut success);
            let o = pa_context_subscribe(context,
                                         PA_SUBSCRIPTION_MASK_SINK,
                                         Some(context_subscribe_cb),
                                         data as *mut _ as *mut libc::c_void);

            if o.is_null() {
                pa_threaded_mainloop_unlock(mainloop);
                bail!("Failed to initialize PA operation!");
            }

            while pa_operation_get_state(o) == PA_OPERATION_RUNNING {
                pa_threaded_mainloop_wait(mainloop);
            }
            pa_operation_unref(o);
            pa_threaded_mainloop_unlock(mainloop);


            let handlers = Handlers::new();
            let cb_box = {
                let h_ref: &Vec<Box<Fn(AudioSignal, AudioUser)>> = &handlers.borrow();
                Box::new((mainloop, h_ref as *const Vec<Box<Fn(AudioSignal, AudioUser)>>))
            };
            {
                pa_context_set_subscribe_callback(context,
                                                  Some(sub_callback),
                                                  Box::into_raw(cb_box) as *mut libc::c_void);

            }


            return Ok(PABackend {
                _cannot_construct: (),
                m: mainloop,
                c: context,
                sink: RefCell::new(sink),
                scroll_step: Cell::new(5),
                handlers,
            })
        }
    }
}


impl AudioFrontend for PABackend {
    // TODO
    fn switch_card(
        &self,
        card_name: Option<String>,
        elem_name: Option<String>,
        user: AudioUser,
    ) -> Result<()> {
        {
            let mut ac = self.sink.borrow_mut();
            *ac = Sink::new(card_name, elem_name, self.m, self.c)?;
        }
        return Ok(())
    }


    fn get_vol(&self) -> Result<f64> {

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

        return vol_to_percent(vol as i64, (PA_VOLUME_MUTED,
                                           PA_VOLUME_NORM))
    }


    fn set_vol(&self, new_vol: f64, user: AudioUser, dir: VolDir, auto_unmute: bool) -> Result<()> {
        let mut res: Result<()> = Err("No value".into());
        let new_vol = percent_to_vol(new_vol, (PA_VOLUME_MUTED,
                                               PA_VOLUME_NORM), dir)?;
        unsafe {
            pa_threaded_mainloop_lock(self.m);
            let data = &mut(self, &mut res);
            let sink_name = CString::new(self.sink.borrow().name.clone()).unwrap().into_raw();

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


    fn vol_level(&self) -> VolLevel {
        let muted = self.get_mute().unwrap_or(false);
        if muted {
            return VolLevel::Muted;
        }
        let cur_vol = try_r!(self.get_vol(), VolLevel::Muted);
        match cur_vol {
            0. => return VolLevel::Off,
            0.0...33.0 => return VolLevel::Low,
            0.0...66.0 => return VolLevel::Medium,
            0.0...100.0 => return VolLevel::High,
            _ => return VolLevel::Off,
        }
    }


    fn increase_vol(&self, user: AudioUser, auto_unmute: bool) -> Result<()> {
        let old_vol = self.get_vol()?;
        let new_vol = old_vol + (self.scroll_step.get() as f64);

        return self.set_vol(new_vol, user, VolDir::Up, auto_unmute);
    }

    fn decrease_vol(&self, user: AudioUser, auto_unmute: bool) -> Result<()> {
        let old_vol = self.get_vol()?;
        let new_vol = old_vol - (self.scroll_step.get() as f64);

        return self.set_vol(new_vol, user, VolDir::Down, auto_unmute);
    }


    fn has_mute(&self) -> bool {
        return true;
    }


    fn get_mute(&self) -> Result<bool> {
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


    fn set_mute(&self, mute: bool, user: AudioUser) -> Result<()> {
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

    fn toggle_mute(&self, user: AudioUser) -> Result<()> {
        let muted = self.get_mute()?;
        return self.set_mute(!muted, user);
    }


    // TODO
    fn connect_handler(&self, cb: Box<Fn(AudioSignal, AudioUser)>) {
        self.handlers.add_handler(cb);
    }

    // TODO: name or desc?
    fn card_name(&self) -> Result<String> {
        return Ok(self.sink.borrow().description.clone())
    }

    fn playable_card_names(&self) -> Vec<String> {
        let sinks = try_r!(get_sinks(self.m, self.c), vec![]);
        return sinks.iter().map(|s| s.description.clone()).collect();
    }

    // TODO
    fn playable_chan_names(&self, cardname: Option<String>) -> Vec<String> {
        return vec![]
    }

    // TODO
    fn chan_name(&self) -> Result<String> {
        return Ok(String::from("Blah"))
    }

    fn set_scroll_step(&self, scroll_step: u32) {
        self.scroll_step.set(scroll_step);
    }

    fn get_scroll_step(&self) -> u32 {
        return self.scroll_step.get();
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
        _: *mut pa_context,
        i: *const pa_sink_info,
        _: i32,
        data: *mut libc::c_void) {
    let (_self, res) = *(data as *mut (*mut PABackend,
                                       *mut u32));
    assert!(!(*_self).m.is_null(), "Mainloop is null");
    assert!(!res.is_null(), "res is null");

    if i.is_null() {
        return
    }

    *res = (*i).volume.values[0];

    pa_threaded_mainloop_signal((*_self).m, 0);
}

// TODO: Better error handling.
unsafe extern "C" fn get_sink_mute(
        _: *mut pa_context,
        i: *const pa_sink_info,
        _: i32,
        data: *mut libc::c_void) {
    let (_self, res) = *(data as *mut (*mut PABackend,
                                       *mut bool));
    assert!(!(*_self).m.is_null(), "Mainloop is null");
    assert!(!res.is_null(), "res is null");

    if i.is_null() {
        return
    }

    *res = (*i).mute != 0;

    pa_threaded_mainloop_signal((*_self).m, 0);
}

// TODO: Missing error handling.
unsafe extern "C" fn set_sink_vol(
        _: *mut pa_context,
        success: i32,
        data: *mut libc::c_void) {
    let (_self, res) = *(data as *mut (*mut PABackend,
                                       *mut Result<()>));
    assert!(!(*_self).m.is_null(), "Mainloop is null");
    assert!(!res.is_null(), "res is null");

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
        _: *mut pa_context,
        success: i32,
        data: *mut libc::c_void) {
    let (_self, res) = *(data as *mut (*mut PABackend,
                                       *mut Result<()>));
    assert!(!(*_self).m.is_null(), "Mainloop is null");
    assert!(!res.is_null(), "res is null");

    if success > 0 {
        *res = Ok(());
    } else {
        *res = Err("Failed to set volume".into());
    }

    pa_threaded_mainloop_signal((*_self).m, 0);
}


unsafe extern "C" fn context_subscribe_cb(c: *mut pa_context,
                                        success: i32,
                                        data: *mut libc::c_void) {
    let (mainloop, res) = *(data as *mut (*mut pa_threaded_mainloop,
                                           *mut bool));

    assert!(!mainloop.is_null(), "Mainloop is null");
    assert!(!res.is_null(), "res is null");

    if success > 0 {
        *res = true;
    } else {
        *res = false;
    }


    pa_threaded_mainloop_signal(mainloop, 0);

}


unsafe extern "C" fn sub_callback(c: *mut pa_context,
                                  t: u32,
                                  idx: u32,
                                  data: *mut libc::c_void) {

    let (mainloop, p_handlers) = *(data as *mut (*mut pa_threaded_mainloop,
                                           *mut Vec<Box<Fn(AudioSignal, AudioUser)>>));

    assert!(!mainloop.is_null(), "Mainloop is null");
    assert!(!p_handlers.is_null(), "Handlers are null");

    let handlers: &Vec<Box<Fn(AudioSignal, AudioUser)>> = &*p_handlers;

    if (t & PA_SUBSCRIPTION_EVENT_FACILITY_MASK) ==
        PA_SUBSCRIPTION_EVENT_SINK {
        if (t & PA_SUBSCRIPTION_EVENT_TYPE_MASK) == PA_SUBSCRIPTION_EVENT_CHANGE {
            // invoke_handlers(
                // handlers,
                // AudioSignal::ValuesChanged,
                // AudioUser::Unknown,
            // );
        }
    }
}

