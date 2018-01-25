#![allow(missing_docs)]

use alsa_lib;
use glib;
use png;
use std::convert::From;
use std;
use toml;



error_chain! {
    foreign_links {
        Alsa(alsa_lib::Error);
        IO(std::io::Error);
        Toml(toml::de::Error);
        Png(png::DecodingError);
        GlibError(glib::Error);
    }

    errors {
        GtkResponseCancel(t: String) {
            description("User hit cancel")
            display("User hit cancel: {}", t)
        }
    }
}




#[macro_export]
/// Try to unwrap a `Result<T, E>`. If there is a value `T`, yield it,
/// otherwise print a warning and `return ()` from the function.
macro_rules! try_w {
    ($expr:expr) => {
        try_wr!($expr, ())
    };
    ($expr:expr, $fmt:expr, $($arg:tt)+) => {
        try_wr!($expr, (), $fmt, $(arg)+)
    };
    ($expr:expr, $fmt:expr) => {
        try_wr!($expr, (), $fmt)
    }
}


#[macro_export]
/// Try to unwrap a `Result<T, E>`. If there is a value `T`, yield it,
/// otherwise print a warning and return from the function with the given value.
macro_rules! try_wr {
    ($expr:expr, $ret:expr) => (match $expr {
        ::std::result::Result::Ok(val) => val,
        ::std::result::Result::Err(err) => {
            warn!("{:?}", err);
            return $ret;
        },
    });
    ($expr:expr, $ret:expr, $fmt:expr) => (match $expr {
        ::std::result::Result::Ok(val) => val,
        ::std::result::Result::Err(err) => {
            warn!("Original error: {:?}", err);
            warn!($fmt);
            return $ret;
        },
    });
    ($expr:expr, $ret:expr, $fmt:expr, $($arg:tt)+) => (match $expr {
        ::std::result::Result::Ok(val) => val,
        ::std::result::Result::Err(err) => {
            warn!("Original error: {:?}", err);
            warn!(format!($fmt, $(arg)+));
            return $ret;
        },
    })
}


#[macro_export]
/// Try to unwrap a `Result<T, E>`. If there is a value `T`, yield it,
/// otherwise return from the function with the given value.
macro_rules! try_r {
    ($expr:expr, $ret:expr) => (match $expr {
        ::std::result::Result::Ok(val) => val,
        ::std::result::Result::Err(_) => {
            return $ret;
        },
    });
}


#[macro_export]
/// Try to unwrap a `Result<T, E>`. If there is a value `T`, yield it,
/// otherwise print an error and exit the program.
macro_rules! try_e {
    ($expr:expr) => (match $expr {
        ::std::result::Result::Ok(val) => val,
        ::std::result::Result::Err(err) => {
            error!("{:?}", err);
            ::std::process::exit(1);
        },
    });
    ($expr:expr, $fmt:expr) => (match $expr {
        ::std::result::Result::Ok(val) => val,
        ::std::result::Result::Err(err) => {
            error!("Original error: {:?}", err);
            error!($fmt);
            std::process::exit(1);
        },
    });
    ($expr:expr, $fmt:expr, $($arg:tt)+) => (match $expr {
        ::std::result::Result::Ok(val) => val,
        ::std::result::Result::Err(err) => {
            error!("Original error: {:?}", err);
            error!(format!($fmt, $(arg)+));
            std::process::exit(1);
        },
    })
}

#[macro_export]
/// Unwraps a `Result<T, E>` by yielding a value of the samet ype
/// for either case.
macro_rules! unwrap_any {
    ($expr:expr, $fmt_ok:expr, $fmt_err:expr) => (match $expr {
        ::std::result::Result::Ok(val) => $fmt_ok,
        ::std::result::Result::Err(err) => $fmt_err,
    })

}


#[macro_export]
/// Warns on err and yields `()` without returning the function.
macro_rules! just_warn {
    ($expr:expr) => (match $expr {
        ::std::result::Result::Ok(_) => (),
        ::std::result::Result::Err(err) => {
            warn!("{:?}", err);
            ()
        },
    });
}


#[macro_export]
/// Present a gtk error dialog with given message.
/// Provides only a close button.
macro_rules! error_dialog {
    ($msg:expr, $parent:expr) => {
        {
            use gtk::DialogExt;
            use gtk::prelude::GtkWindowExt;
            use gtk::WidgetExt;

            let parent: Option<&gtk::Window> = $parent;
            let dialog = gtk::MessageDialog::new(parent,
                                                 gtk::DialogFlags::DESTROY_WITH_PARENT,
                                                 gtk::MessageType::Error,
                                                 gtk::ButtonsType::Close,
                                                 $msg);
            dialog.set_title("PNMixer-rs Error");

            dialog.run();
            dialog.destroy();
        }
    };
}


#[macro_export]
/// Present a gtk error dialog with the error from the `Result` type,
/// if any.
/// Provides only a close button.
macro_rules! result_warn {
    ($expr:expr, $parent:expr) => (match $expr {
        ::std::result::Result::Ok(v) => Ok(v),
        ::std::result::Result::Err(err) => {
            use std::error::Error;
            let warn_string = format!("{}{}", err.description(),
                err.cause().map(|e| format!("\n\nCause: {}", e.description())).unwrap_or(String::from("")));
            warn!("{}", warn_string);
            error_dialog!(warn_string.as_str(), $parent);
            Err(err)
        },
    });
}


#[macro_export]
/// Convert `WResult` to `Result`. All warnings are printed via the `log`
/// crate and are shown via Gtk dialogs.
macro_rules! wresult_warn {
    ($expr:expr, $parent:expr) => (match $expr {
        ::w_result::WResult::WOk(t, ws) => {
            use std::error::Error;
            for w in ws {
                let warn_string = format!("{}{}", w.description(),
                    w.cause().map(|e| format!("\n\nCause: {}", e.description())).unwrap_or(String::from("")));
                warn!("{}", warn_string);
                error_dialog!(warn_string.as_str(), $parent);
            }
            Ok(t)
        },
        ::w_result::WResult::WErr(err) => Err(err),
    });
}


#[macro_export]
/// If there is an error in the expression, push it to
/// the given mutable warning vector.
macro_rules! push_warning {
    ($expr:expr, $vec:ident) => (match $expr {
            Err(e) => $vec.push(e),
            _ => ()
    });
}


#[macro_export]
/// If there is a value in the Result type, unwrap it, otherwise error-log
/// the error, show it via gtk dialog and exit the whole program.
macro_rules! unwrap_error {
    ($expr:expr, $parent:expr) => (match $expr {
        ::std::result::Result::Ok(v) => v,
        ::std::result::Result::Err(err) => {
            use std::error::Error;
            let err_string = format!("{}{}", err.description(),
                err.cause().map(|e| format!("\n\nCause: {}", e.description())).unwrap_or(String::from("")));

            error!("{}", err_string);
            error_dialog!(err_string.as_str(), $parent);
            ::std::process::exit(1);
        },
    });
}
