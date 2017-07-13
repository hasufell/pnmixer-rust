use errors::*;
use gdk_pixbuf;
use gdk_pixbuf_sys;
use glib::translate::FromGlibPtrFull;
use glib::translate::ToGlibPtr;
use gtk::prelude::*;
use gtk;
use std::path::*;



pub fn copy_pixbuf(pixbuf: &gdk_pixbuf::Pixbuf) -> gdk_pixbuf::Pixbuf {

    let new_pixbuf = unsafe {
        let gdk_pixbuf = pixbuf.to_glib_full();
        let copy = gdk_pixbuf_sys::gdk_pixbuf_copy(gdk_pixbuf);
        FromGlibPtrFull::from_glib_full(copy)
    };

    return new_pixbuf;
}


pub fn pixbuf_new_from_theme(icon_name: &str,
                             size: i32,
                             theme: &gtk::IconTheme)
                             -> Result<gdk_pixbuf::Pixbuf> {

    let icon_info =
        theme.lookup_icon(icon_name, size, gtk::IconLookupFlags::empty())
            .ok_or(format!("Couldn't find icon {}", icon_name))?;

    debug!("Loading stock icon {} from {:?}",
           icon_name,
           icon_info.get_filename().unwrap_or(PathBuf::new()));

    // TODO: propagate error
    let pixbuf = icon_info.load_icon().unwrap();

    return Ok(pixbuf);
}


#[macro_export]
macro_rules! pixbuf_new_from_file {
    ($name:expr) => {
        {
            use gdk_pixbuf;
            use png;

            let bytes = include_bytes!($name);
            let pixbuf_new_from_bytes = |bytes| -> Result<gdk_pixbuf::Pixbuf> {
                let decoder = png::Decoder::new(bytes);
                let (info, mut reader) = decoder.read_info()?;
                let mut buf = vec![0; info.buffer_size()];
                reader.next_frame(&mut buf).unwrap();

                ensure!(info.color_type == png::ColorType::RGB ||
                        info.color_type == png::ColorType::RGBA,
                        "Only RGB is supported for GDKPixbuf");

                debug!("Loading icon from {}", $name);

                return Ok(gdk_pixbuf::Pixbuf::new_from_vec(buf,
                                                 gdk_pixbuf_sys::GDK_COLORSPACE_RGB,
                                                 true,
                                                 info.bit_depth as i32,
                                                 info.width as i32,
                                                 info.height as i32,
                                                 info.line_size as i32));
            };
            pixbuf_new_from_bytes(bytes as &[u8])
        }
    }
}

