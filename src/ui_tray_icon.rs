use app_state::*;
use audio::*;
use errors::*;
use glib;
use gio;
use gdk;
use gdk_pixbuf;
use gdk_pixbuf_sys;
use gtk::prelude::*;
use gtk;
use prefs::{Prefs, MiddleClickAction};
use std::cell::Cell;
use std::cell::RefCell;
use std::rc::Rc;
use support_cmd::*;
use support_ui::*;
use ui_prefs_dialog::show_prefs_dialog;


// TODO tooltip


const ICON_MIN_SIZE: i32 = 16;



pub struct TrayIcon {
    _cant_construct: (),
    pub volmeter: RefCell<Option<VolMeter>>,
    pub audio_pix: RefCell<AudioPix>,
    pub status_icon: gtk::StatusIcon,
    pub icon_size: Cell<i32>,
}


impl TrayIcon {
    pub fn new(prefs: &Prefs) -> Result<TrayIcon> {
        let draw_vol_meter = prefs.view_prefs.draw_vol_meter;

        let volmeter = {
            if draw_vol_meter {
                RefCell::new(Some(VolMeter::new(prefs)))
            } else {
                RefCell::new(None)
            }
        };
        let audio_pix = AudioPix::new(ICON_MIN_SIZE, prefs)?;
        let status_icon = gtk::StatusIcon::new();

        return Ok(TrayIcon {
                      _cant_construct: (),
                      volmeter,
                      audio_pix: RefCell::new(audio_pix),
                      status_icon,
                      icon_size: Cell::new(ICON_MIN_SIZE),
                  });
    }


    fn update_pixbuf(&self, cur_vol: f64, vol_level: VolLevel) -> Result<()> {
        let audio_pix = self.audio_pix.borrow();
        let pixbuf = audio_pix.select_pix(vol_level);

        let vol_borrow = self.volmeter.borrow();
        let volmeter = &vol_borrow.as_ref();
        match volmeter {
            &Some(v) => {
                let vol_pix = v.meter_draw(cur_vol as i64, &pixbuf)?;
                self.status_icon.set_from_pixbuf(Some(&vol_pix));
            }
            &None => self.status_icon.set_from_pixbuf(Some(pixbuf)),
        };

        return Ok(());
    }


    fn update_audio(&self, audio: &Audio) -> Result<()> {

        return self.update_pixbuf(audio.vol()?, audio.vol_level());
    }


    fn update_tooltip(&self, audio: &Audio) {
        let cardname = audio.acard
            .borrow()
            .card_name()
            .unwrap_or(String::from("Unknown card"));
        let channame = audio.acard
            .borrow()
            .chan_name()
            .unwrap_or(String::from("unknown channel"));
        let vol = audio.vol()
            .map(|s| format!("{}", s.round()))
            .unwrap_or(String::from("unknown volume"));
        let mute_info = {
            if !audio.has_mute() {
                "\nNo mute switch"
            } else if audio.get_mute().unwrap_or(false) {
                "\nMuted"
            } else {
                ""
            }
        };
        self.status_icon.set_tooltip_text(format!("{} ({})\nVolume: {}{}",
                                                  cardname,
                                                  channame,
                                                  vol,
                                                  mute_info)
                                                  .as_str());
    }


    pub fn update_all(&self,
                      prefs: &Prefs,
                      audio: &Audio,
                      m_size: Option<i32>)
                      -> Result<()> {
        match m_size {
            Some(s) => {
                if s < ICON_MIN_SIZE {
                    self.icon_size.set(ICON_MIN_SIZE);

                } else {
                    self.icon_size.set(s);
                }
            }
            None => (),
        }

        let audio_pix = AudioPix::new(self.icon_size.get(), &prefs)?;
        *self.audio_pix.borrow_mut() = audio_pix;

        let draw_vol_meter = prefs.view_prefs.draw_vol_meter;
        if draw_vol_meter {
            let volmeter = VolMeter::new(&prefs);
            *self.volmeter.borrow_mut() = Some(volmeter);
        }

        self.update_tooltip(&audio);
        return self.update_pixbuf(audio.vol()?, audio.vol_level());
    }
}



pub struct VolMeter {
    red: u8,
    green: u8,
    blue: u8,
    x_offset_pct: i64,
    y_offset_pct: i64,
    /* dynamic */
    width: Cell<i64>,
    row: RefCell<Vec<u8>>,
}


impl VolMeter {
    fn new(prefs: &Prefs) -> VolMeter {
        return VolMeter {
                   red: (prefs.view_prefs.vol_meter_color.red * 255.0) as u8,
                   green: (prefs.view_prefs.vol_meter_color.green * 255.0) as u8,
                   blue: (prefs.view_prefs.vol_meter_color.blue * 255.0) as u8,
                   x_offset_pct: prefs.view_prefs.vol_meter_offset as i64,
                   y_offset_pct: 10,
                   /* dynamic */
                   width: Cell::new(0),
                   row: RefCell::new(vec![]),
               };
    }

    // TODO: cache input pixbuf?
    fn meter_draw(&self,
                  volume: i64,
                  pixbuf: &gdk_pixbuf::Pixbuf)
                  -> Result<gdk_pixbuf::Pixbuf> {

        ensure!(pixbuf.get_colorspace() == gdk_pixbuf_sys::GDK_COLORSPACE_RGB,
                "Invalid colorspace in pixbuf");
        ensure!(pixbuf.get_bits_per_sample() == 8,
                "Invalid bits per sample in pixbuf");
        ensure!(pixbuf.get_has_alpha(), "No alpha channel in pixbuf");
        ensure!(pixbuf.get_n_channels() == 4,
                "Invalid number of channels in pixbuf");

        let i_width = pixbuf.get_width() as i64;
        let i_height = pixbuf.get_height() as i64;

        let new_pixbuf = copy_pixbuf(pixbuf);

        let vm_width = i_width / 6;
        let x = (self.x_offset_pct as f64 *
                 ((i_width - vm_width) as f64 / 100.0)) as i64;
        ensure!(x >= 0 && (x + vm_width) <= i_width,
                "x coordinate invalid: {}",
                x);
        let y = (self.y_offset_pct as f64 * (i_height as f64 / 100.0)) as i64;
        let vm_height =
            ((i_height - (y * 2)) as f64 * (volume as f64 / 100.0)) as i64;
        ensure!(y >= 0 && (y + vm_height) <= i_height,
                "y coordinate invalid: {}",
                y);

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
                pixels[p_index..p_index + row.len()]
                    .copy_from_slice(row.as_ref());

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
    fn new(size: i32, prefs: &Prefs) -> Result<AudioPix> {
        let system_theme = prefs.view_prefs.system_theme;

        let pix = {
            if system_theme {
                let theme: gtk::IconTheme =
                    gtk::IconTheme::get_default().ok_or(
                        "Couldn't get default icon theme",
                    )?;
                AudioPix {
                    muted: pixbuf_new_from_theme(
                        "audio-volume-muted",
                        size,
                        &theme,
                    )?,
                    low: pixbuf_new_from_theme(
                        "audio-volume-low",
                        size,
                        &theme,
                    )?,
                    medium: pixbuf_new_from_theme(
                        "audio-volume-medium",
                        size,
                        &theme,
                    )?,
                    high: pixbuf_new_from_theme(
                        "audio-volume-high",
                        size,
                        &theme,
                    )?,
                    /* 'audio-volume-off' is not available in every icon set.
                     * Check freedesktop standard for more info:
                     *   http://standards.freedesktop.org/icon-naming-spec/
                     *   icon-naming-spec-latest.html
                     */
                    off: pixbuf_new_from_theme(
                        "audio-volume-off",
                        size,
                        &theme,
                    ).or(pixbuf_new_from_theme(
                        "audio-volume-low",
                        size,
                        &theme,
                    ))?,
                }
            } else {
                AudioPix {
                    muted: pixbuf_new_from_file("pnmixer-muted.png")?,
                    low: pixbuf_new_from_file("pnmixer-low.png")?,
                    medium: pixbuf_new_from_file("pnmixer-medium.png")?,
                    high: pixbuf_new_from_file("pnmixer-high.png")?,
                    off: pixbuf_new_from_file("pnmixer-off.png")?,
                }
            }
        };
        return Ok(pix);
    }


    fn select_pix(&self, vol_level: VolLevel) -> &gdk_pixbuf::Pixbuf {
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
    try_e!(tray_icon.update_all(&appstate.prefs.borrow_mut(), &audio, None));

    tray_icon.status_icon.set_visible(true);

    /* connect audio handler */
    {
        let apps = appstate.clone();
        appstate.audio.connect_handler(Box::new(move |s, u| match (s, u) {
                                                    (_, _) => {
            apps.gui.tray_icon.update_tooltip(&apps.audio);
            try_w!(apps.gui.tray_icon.update_audio(&apps.audio));
        }
                                                }));
    }

    /* tray_icon.connect_size_changed */
    {
        let apps = appstate.clone();
        tray_icon.status_icon.connect_size_changed(move |_, size| {
            try_wr!(apps.gui.tray_icon.update_all(&apps.prefs.borrow_mut(),
                                                  &apps.audio,
                                                  Some(size)),
                    false);
            return false;
        });
    }

    /* tray_icon.connect_activate */
    {
        let apps = appstate.clone();
        tray_icon.status_icon.connect_activate(move |_| {
                                                   on_tray_icon_activate(&apps)
                                               });
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
        tray_icon.status_icon.connect_popup_menu(move |_, _, _| {
            on_tray_icon_popup_menu(&apps)
        });
    }

    /* tray_icon.connect_button_release_event */
    {
        let apps = appstate.clone();
        tray_icon.status_icon.connect_button_release_event(move |_, eb| {
            on_tray_button_release_event(&apps, eb)
        });
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


fn on_tray_icon_scroll_event(appstate: &AppS,
                             event: &gdk::EventScroll)
                             -> bool {

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


fn on_tray_button_release_event(appstate: &Rc<AppS>,
                                event_button: &gdk::EventButton)
                                -> bool {
    let button = event_button.get_button();

    if button != 2 {
        // not middle-click
        return false;
    }

    let audio = &appstate.audio;
    let prefs = &appstate.prefs.borrow();
    let middle_click_action = &prefs.behavior_prefs.middle_click_action;
    let custom_command = &prefs.behavior_prefs.custom_command;

    match middle_click_action {
        &MiddleClickAction::ToggleMute => {
            if audio.has_mute() {
                try_wr!(audio.toggle_mute(AudioUser::Popup), false);
            }
        }
        // TODO
        &MiddleClickAction::ShowPreferences => show_prefs_dialog(&appstate),
        &MiddleClickAction::VolumeControl => {
            try_wr!(execute_vol_control_command(&appstate.prefs.borrow()),
                    false);
        }
        &MiddleClickAction::CustomCommand => {
            match custom_command {
                &Some(ref cmd) => try_wr!(execute_command(cmd.as_str()), false),
                &None => warn!("No custom command found"),
            }
        }
    }


    return false;
}
