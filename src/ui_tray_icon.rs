use app_state::*;
use audio::*;
use errors::*;
use gdk;
use gdk_pixbuf;
use gdk_pixbuf_sys::GDK_COLORSPACE_RGB;
use gtk::prelude::*;
use gtk;
use std::cell::Cell;
use std::cell::RefCell;
use std::rc::Rc;
use support_ui::*;


// TODO: on_apply


const ICON_MIN_SIZE: i64 = 16;



pub struct TrayIcon {
    pub volmeter: VolMeter,
    pub audio_pix: AudioPix,
    pub status_icon: gtk::StatusIcon,
    pub icon_size: Cell<i64>,
}


impl TrayIcon {
    // TODO: take settings as parameter
    pub fn new() -> Result<TrayIcon> {
        let volmeter = VolMeter::new();
        let audio_pix = AudioPix::new_from_pnmixer()?;
        let status_icon = gtk::StatusIcon::new();

        return Ok(TrayIcon { volmeter, audio_pix, status_icon, icon_size: Cell::new(ICON_MIN_SIZE) });
    }

    fn update(&self, audio: &Audio, m_size: Option<i64>) {
        match m_size {
            Some(s) => {
                if s < ICON_MIN_SIZE {
                    self.icon_size.set(ICON_MIN_SIZE);
                } else {
                    self.icon_size.set(s);
                }
            },
            None => (),
        }

        let cur_vol = try_w!(audio.vol());
        let pixbuf = self.audio_pix.select_pix(audio.vol_level());
        let vol_pix = try_w!(self.volmeter.meter_draw(cur_vol as i64,
                                                      &pixbuf));

        self.status_icon.set_from_pixbuf(Some(&vol_pix));
    }
}



pub struct VolMeter {
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
                .take((vm_width * 4) as usize)
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
                let p_index = ((row_offset * rowstride) + col_offset) as usize;

                let row = self.row.borrow();
                pixels[p_index..p_index + row.len()].copy_from_slice(
                    row.as_ref(),
                );

            }
        }

        return Ok(new_pixbuf);
    }
}


// TODO: connect on icon theme change


#[derive(Clone, Debug)]
pub struct AudioPix {
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


pub fn init_tray_icon(appstate: Rc<AppS>) {
    let audio = &appstate.audio;
    let tray_icon = &appstate.gui.tray_icon;
    tray_icon.update(&audio, None);

    tray_icon.status_icon.set_visible(true);

    /* connect audio handler */
    {
        let apps = appstate.clone();
        appstate.audio.connect_handler(
            Box::new(move |s, u| match (s, u) {
                (AudioSignal::ValuesChanged, _) => {
                    apps.gui.tray_icon.update(&apps.audio, None);
                }
                _ => (),
            }),
        );
    }

    /* tray_icon.connect_size_changed */
    {
        let apps = appstate.clone();
        tray_icon.status_icon.connect_size_changed(move |_, size| {
            apps.gui.tray_icon.update(&apps.audio, Some(size as i64));
            return false;
        });
    }

    /* tray_icon.connect_activate */
    {
        let apps = appstate.clone();
        tray_icon.status_icon.connect_activate(move |_| on_tray_icon_activate(&apps));
    }

    /* tray_icon.connect_scroll_event */
    {
        let apps = appstate.clone();
        tray_icon.status_icon.connect_scroll_event(
            move |_, e| on_tray_icon_scroll_event(&apps, &e),
        );
    }

    /* tray_icon.connect_popup_menu */
    {
        let apps = appstate.clone();
        tray_icon.status_icon.connect_popup_menu(
            move |_, _, _| on_tray_icon_popup_menu(&apps),
        );
    }

    /* tray_icon.connect_button_release_event */
    {
        let apps = appstate.clone();
        tray_icon.status_icon.connect_button_release_event(
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
