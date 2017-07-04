use app_state::*;
use gdk;
use gdk_pixbuf;
use gdk_pixbuf_sys;
use gdk_pixbuf_sys::GDK_COLORSPACE_RGB;
use gdk_sys;
use glib;
use glib_sys;
use gtk;
use std::mem;
use gtk::prelude::*;
use std::rc::Rc;
use std::cell::Cell;
use std::cell::RefCell;
use libc;
use audio::*;
use errors::*;
use std::path::*;
use glib::translate::ToGlibPtr;
use glib::translate::FromGlibPtrFull;
use glib::translate::FromGlibPtrNone;

use libc::memcpy;



const ICON_MIN_SIZE: i32 = 16;


fn copy_pixbuf(pixbuf: &gdk_pixbuf::Pixbuf) -> gdk_pixbuf::Pixbuf {

    let new_pixbuf = unsafe {
        let gdk_pixbuf = pixbuf.to_glib_full();
        let copy = gdk_pixbuf_sys::gdk_pixbuf_copy(gdk_pixbuf);
        FromGlibPtrFull::from_glib_full(copy)
    };

    return new_pixbuf;
}


struct VolMeter {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
    pub x_offset_pct: i64,
    pub y_offset_pct: i64,
    /* dynamic */
    pub width: Cell<i64>,
    pub row: RefCell<Vec<u8>>,
}


impl VolMeter {
    // TODO: take settings
    pub fn new() -> VolMeter {
        return VolMeter {
            red: 245,
            green: 121,
            blue: 0,
            x_offset_pct: 10,
            y_offset_pct: 10,
            /* dynamic */
            width: Cell::new(0),
            row: RefCell::new(vec![]),
        };
    }

    // TODO: cache input pixbuf?
    fn meter_draw(
        &self,
        volume: i64,
        pixbuf: &gdk_pixbuf::Pixbuf,
    ) -> Result<gdk_pixbuf::Pixbuf> {

        ensure!(
            pixbuf.get_colorspace() == GDK_COLORSPACE_RGB,
            "Invalid colorspace in pixbuf"
        );
        ensure!(
            pixbuf.get_bits_per_sample() == 8,
            "Invalid bits per sample in pixbuf"
        );
        ensure!(pixbuf.get_has_alpha(), "No alpha channel in pixbuf");
        ensure!(
            pixbuf.get_n_channels() == 4,
            "Invalid number of channels in pixbuf"
        );

        let i_width = pixbuf.get_width() as i64;
        let i_height = pixbuf.get_height() as i64;

        let new_pixbuf = copy_pixbuf(pixbuf);

        let vm_width = i_width / 6;
        let x = (self.x_offset_pct as f64 *
                     ((i_width - vm_width) as f64 / 100.0)) as
            i64;
        ensure!(
            x >= 0 && (x + vm_width) <= i_width,
            "x coordinate invalid: {}",
            x
        );
        let y = (self.y_offset_pct as f64 * (i_height as f64 / 100.0)) as i64;
        let vm_height =
            ((i_height - (y * 2)) as f64 * (volume as f64 / 100.0)) as i64;
        ensure!(
            y >= 0 && (y + vm_height) <= i_height,
            "y coordinate invalid: {}",
            y
        );

        debug!("vm_height: {}", vm_height);
        debug!("i_height: {}", i_height);
        debug!("y: {}", y);
        debug!("volume: {}", volume);

        /* Let's check if the icon width changed, in which case we
         * must reinit our internal row of pixels.
         */
        if vm_width != self.width.get() {
            self.width.set(vm_width);
            let mut row = self.row.borrow_mut();
            *row = vec![];
        }

        if self.row.borrow().len() == 0 {
            debug!("Allocating vol meter row (width {})", vm_width);
            let mut row = self.row.borrow_mut();
            *row = [self.red, self.green, self.blue, 255]
                .iter()
                .cloned()
                .cycle()
                .take(vm_width - 1 as usize)
                .collect();
        }

        /* Draw the volume meter.
         * Rows in the image are stored top to bottom.
         */
        {
            let y = i_height - y;
            let rowstride: i64 = new_pixbuf.get_rowstride() as i64;
            let pixels: &mut [u8] = unsafe { new_pixbuf.get_pixels() };

            for i in 0..(vm_height - 1) {
                let row_offset: i64 = y - i;
                let col_offset: i64 = x * 4;
                let p_index = ((row_offset * rowstride) + col_offset) as isize;

                unsafe {
                    let p = pixels.as_mut_ptr().offset(p_index) as
                        *mut libc::c_void;
                    memcpy(
                        p,
                        self.row.borrow().as_slice().as_ptr() as
                            *const libc::c_void,
                        self.row.borrow().len(),
                    );
                }
            }
        }

        return Ok(new_pixbuf);
    }
}


// TODO: connect on icon theme change


#[derive(Clone, Debug)]
struct AudioPix {
    muted: gdk_pixbuf::Pixbuf,
    low: gdk_pixbuf::Pixbuf,
    medium: gdk_pixbuf::Pixbuf,
    high: gdk_pixbuf::Pixbuf,
    off: gdk_pixbuf::Pixbuf,
}


impl AudioPix {
    // TODO: take settings
    pub fn new_from_theme(size: i32) -> Result<AudioPix> {
        let theme: gtk::IconTheme = gtk::IconTheme::get_default().ok_or(
            "Couldn't get default icon theme",
        )?;
        let pix = AudioPix {
            muted: pixbuf_new_from_theme("audio-volume-muted", size, &theme)?,
            low: pixbuf_new_from_theme("audio-volume-low", size, &theme)?,
            medium: pixbuf_new_from_theme("audio-volume-medium", size, &theme)?,
            high: pixbuf_new_from_theme("audio-volume-high", size, &theme)?,
            /* 'audio-volume-off' is not available in every icon set.
             * Check freedesktop standard for more info:
             *   http://standards.freedesktop.org/icon-naming-spec/
             *   icon-naming-spec-latest.html
             */
            off: pixbuf_new_from_theme("audio-volume-off", size, &theme).or(
                pixbuf_new_from_theme("audio-volume-low", size, &theme),
            )?,
        };

        return Ok(pix);
    }

    pub fn new_from_pnmixer() -> Result<AudioPix> {
        gtk::IconTheme::get_default().ok_or(
            "Couldn't get default icon theme",
        )?;
        let pix = AudioPix {
            muted: pixbuf_new_from_file("pnmixer-muted.png")?,
            low: pixbuf_new_from_file("pnmixer-low.png")?,
            medium: pixbuf_new_from_file("pnmixer-medium.png")?,
            high: pixbuf_new_from_file("pnmixer-high.png")?,
            off: pixbuf_new_from_file("pnmixer-off.png")?,
        };

        return Ok(pix);
    }

    pub fn select_pix(&self, vol_level: VolLevel) -> &gdk_pixbuf::Pixbuf {
        match vol_level {
            VolLevel::Muted => &self.muted,
            VolLevel::Low => &self.low,
            VolLevel::Medium => &self.medium,
            VolLevel::High => &self.high,
            VolLevel::Off => &self.off,
        }
    }
}


fn pixbuf_new_from_theme(
    icon_name: &str,
    size: i32,
    theme: &gtk::IconTheme,
) -> Result<gdk_pixbuf::Pixbuf> {

    let icon_info = theme
        .lookup_icon(icon_name, size, gtk::IconLookupFlags::empty())
        .ok_or(format!("Couldn't find icon {}", icon_name))?;

    debug!(
        "Loading stock icon {} from {:?}",
        icon_name,
        icon_info.get_filename().unwrap_or(PathBuf::new())
    );

    // TODO: propagate error
    let pixbuf = icon_info.load_icon().unwrap();

    return Ok(pixbuf);
}


fn pixbuf_new_from_file(filename: &str) -> Result<gdk_pixbuf::Pixbuf> {
    ensure!(!filename.is_empty(), "Filename is empty");

    let s = format!("./data/pixmaps/{}", filename);
    let path = Path::new(s.as_str());

    if path.exists() {
        let str_path = path.to_str().ok_or("Path is not valid unicode")?;

        // TODO: propagate error
        return Ok(gdk_pixbuf::Pixbuf::new_from_file(str_path).unwrap());
    } else {
        bail!("Uh-oh");
    }
}


fn update_tray_icon(audio_pix: &AudioPix, appstate: &AppS) {
    let cur_vol = try_w!(appstate.audio.vol());

    let status_icon = &appstate.gui.status_icon;
    let pixbuf = audio_pix.select_pix(appstate.audio.vol_level());

    let volmeter = VolMeter::new();
    let vol_pix = try_w!(volmeter.meter_draw(cur_vol as i64, &pixbuf));

    status_icon.set_from_pixbuf(Some(&vol_pix));
}


pub fn init_tray_icon(appstate: Rc<AppS>) {
    let audio_pix = Rc::new(RefCell::new(try_w!(AudioPix::new_from_pnmixer())));
    update_tray_icon(&audio_pix.borrow(), &appstate);

    /* connect audio handler */
    {
        let _audio_pix = audio_pix.clone();
        let apps = appstate.clone();
        appstate.audio.connect_handler(
            Box::new(move |s, u| match (s, u) {
                (AudioSignal::ValuesChanged, _) => {
                    update_tray_icon(&_audio_pix.borrow(), &apps);
                }
                _ => (),
            }),
        );
    }

    /* tray_icon.connect_size_changed */
    {
        let apps = appstate.clone();
        let tray_icon = &appstate.gui.status_icon;
        let _audio_pix = audio_pix.clone();
        tray_icon.connect_size_changed(move |_, size| {
            on_tray_icon_size_changed(&apps, _audio_pix.as_ref(), size)
        });
        tray_icon.set_visible(true);
    }

    /* tray_icon.connect_activate */
    {
        let apps = appstate.clone();
        let tray_icon = &appstate.gui.status_icon;
        tray_icon.connect_activate(move |_| on_tray_icon_activate(&apps));
        tray_icon.set_visible(true);
    }

    /* tray_icon.connect_scroll_event */
    {
        let apps = appstate.clone();
        let tray_icon = &appstate.clone().gui.status_icon;
        tray_icon.connect_scroll_event(
            move |_, e| on_tray_icon_scroll_event(&apps, &e),
        );
    }

    /* tray_icon.connect_popup_menu */
    {
        let apps = appstate.clone();
        let tray_icon = &appstate.clone().gui.status_icon;
        tray_icon.connect_popup_menu(
            move |_, _, _| on_tray_icon_popup_menu(&apps),
        );
    }

    /* tray_icon.connect_button_release_event */
    {
        let apps = appstate.clone();
        let tray_icon = &appstate.clone().gui.status_icon;
        tray_icon.connect_button_release_event(
            move |_, eb| on_tray_button_release_event(&apps, eb),
        );
    }
}


fn on_tray_icon_activate(appstate: &AppS) {
    let popup_window = &appstate.gui.popup_window.popup_window;

    if popup_window.get_visible() {
        popup_window.hide();
    } else {
        popup_window.show_now();
    }
}


fn on_tray_icon_popup_menu(appstate: &AppS) {
    let popup_window = &appstate.gui.popup_window.popup_window;
    let popup_menu = &appstate.gui.popup_menu.menu;

    popup_window.hide();
    popup_menu.popup_at_pointer(None);
}


fn on_tray_icon_scroll_event(
    appstate: &AppS,
    event: &gdk::EventScroll,
) -> bool {

    let audio = &appstate.audio;

    let scroll_dir: gdk::ScrollDirection = event.get_direction();
    match scroll_dir {
        gdk::ScrollDirection::Up => {
            try_wr!(appstate.audio.increase_vol(AudioUser::TrayIcon), false);
        }
        gdk::ScrollDirection::Down => {
            try_wr!(appstate.audio.decrease_vol(AudioUser::TrayIcon), false);
        }
        _ => (),
    }

    return false;
}


fn on_tray_icon_size_changed(
    appstate: &AppS,
    audio_pix: &RefCell<AudioPix>,
    size: i32,
) -> bool {
    debug!("Tray icon size is now {}", size);

    let mut size = size;
    if size < ICON_MIN_SIZE {
        size = ICON_MIN_SIZE;
        debug!("Forcing size to the minimum value {}", size);
    }

    {
        let mut pix = audio_pix.borrow_mut();
        *pix = try_wr!(AudioPix::new_from_pnmixer(), false);
    }

    update_tray_icon(&audio_pix.borrow(), &appstate);

    return false;
}


fn on_tray_button_release_event(
    appstate: &AppS,
    event_button: &gdk::EventButton,
) -> bool {
    let button = event_button.get_button();

    if button != 2 {
        // not middle-click
        return false;
    }

    let audio = &appstate.audio;
    try_wr!(audio.toggle_mute(AudioUser::Popup), false);

    return false;
}
