//! Glue code between gdk and x11, allowing some `gdk_x11_*` functions.
//!
//! This is not a complete binding, but just provides what we need in a
//! reasonable way.


use gdk;
use gdk_sys::GdkDisplay;
use glib::translate::*;
use x11::xlib::{Display, Window};


// https://developer.gnome.org/gdk3/stable/gdk3-X-Window-System-Interaction.html
mod ffi {
    use gdk_sys::{GdkDisplay, GdkWindow};
    use x11::xlib::{Display, Window};

    extern "C" {
        pub fn gdk_x11_get_default_xdisplay() -> *mut Display;
        pub fn gdk_x11_get_default_root_xwindow() -> Window;
        pub fn gdk_x11_window_foreign_new_for_display(
            display: *mut GdkDisplay,
            window: Window,
        ) -> *mut GdkWindow;
    }
}


/// Gets the default GTK+ display.
///
/// # Returns
///
/// the Xlib Display* for the display specified in the `--display`
/// command line option or the `DISPLAY` environment variable.
pub fn gdk_x11_get_default_xdisplay() -> *mut Display {
    unsafe {
        return ffi::gdk_x11_get_default_xdisplay();
    }
}


/// Gets the root window of the default screen (see `gdk_x11_get_default_screen()`).
///
/// # Returns
///
/// an Xlib Window.
pub fn gdk_x11_get_default_root_xwindow() -> Window {
    unsafe {
        return ffi::gdk_x11_get_default_root_xwindow();
    }
}


/// Wraps a native window in a GdkWindow. The function will try to look up the
/// window using `gdk_x11_window_lookup_for_display()` first. If it does not find
/// it there, it will create a new window.
///
/// This may fail if the window has been destroyed. If the window was already
/// known to GDK, a new reference to the existing GdkWindow is returned.
/// ## `display`
/// the GdkDisplay where the window handle comes from.
/// ## ` window`
/// an Xlib Window
///
/// # Returns
///
/// a GdkWindow wrapper for the native window, or `None` if the window has been
/// destroyed. The wrapper will be newly created, if one doesn’t exist already.
pub fn gdk_x11_window_foreign_new_for_display(
    gdk_display: &mut gdk::Display,
    xwindow: Window,
) -> Option<gdk::Window> {
    unsafe {
        let display: *mut GdkDisplay =
            mut_override(gdk_display.to_glib_none().0);

        return from_glib_full(ffi::gdk_x11_window_foreign_new_for_display(
            display,
            xwindow,
        ));
    }
}
