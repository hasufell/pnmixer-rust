use audio::pulseaudio::Sink;
use errors::*;
use libc;
use libpulse_sys::*;
use std::ffi::CStr;
use std::mem;


pub fn get_sinks(mainloop: *mut pa_threaded_mainloop,
                 ctx: *mut pa_context) -> Result<Vec<Sink>> {
    unsafe {
        let mut v = vec![];
        {
            pa_threaded_mainloop_lock(mainloop);
            let _data = &mut(mainloop, &mut v);
            let data = mem::transmute::<&mut(*mut pa_threaded_mainloop,
                                             &mut Vec<Sink>),
                                             *mut libc::c_void>(_data);
            let o = pa_context_get_sink_info_list(ctx,
                                                  Some(get_all_sinks),
                                                  data);
            if o.is_null() {
                pa_threaded_mainloop_unlock(mainloop);
                bail!("Failed to initialize PA operation!");
            }

            while pa_operation_get_state(o) == PA_OPERATION_RUNNING {
                pa_threaded_mainloop_wait(mainloop);
            }
            pa_operation_unref(o);
            pa_threaded_mainloop_unlock(mainloop);
        }

        return Ok(v);
    }
}

unsafe extern "C" fn get_all_sinks(
        ctx: *mut pa_context,
        i: *const pa_sink_info,
        eol: i32,
        data: *mut libc::c_void) {
    let &mut(mainloop, vec) = mem::transmute::<*mut libc::c_void,
                                        &mut(*mut pa_threaded_mainloop,
                                            *mut Vec<Sink>)>(data);
    assert!(!mainloop.is_null(), "Mainloop is null");

    if i.is_null() {
        return
    }
    let name = CStr::from_ptr((*i).name).to_str().unwrap().to_owned();
    let index = (*i).index;
    let description = CStr::from_ptr((*i).description).to_str().unwrap().to_owned();
    let channels = (*i).channel_map.channels;

    (*vec).push(Sink {
        name,
        index,
        description,
        channels,
    });
    pa_threaded_mainloop_signal(mainloop, 0);
}

pub fn get_first_sink(mainloop: *mut pa_threaded_mainloop,
                      ctx: *mut pa_context) -> Result<Sink> {
    let sinks = get_sinks(mainloop, ctx)?;
    ensure!(!sinks.is_empty(), "No sinks found!");

    return Ok(sinks[0].clone());
}

// TODO: Could be done directly via PA API.
pub fn get_sink_by_desc(mainloop: *mut pa_threaded_mainloop,
                        ctx: *mut pa_context,
                        desc: &str) -> Result<Sink> {
    let sinks = get_sinks(mainloop, ctx)?;
    ensure!(!sinks.is_empty(), "No sinks found!");

    return Ok(sinks.into_iter()
              .find(|s| s.description == desc)
              .map(|s| s.clone())
              .ok_or("No sink found with desc")?);
}
