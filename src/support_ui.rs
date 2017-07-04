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


pub fn pixbuf_new_from_theme(
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


pub fn pixbuf_new_from_file(filename: &str) -> Result<gdk_pixbuf::Pixbuf> {
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


