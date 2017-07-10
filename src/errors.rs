use alsa;
use std::convert::From;
use std;
use toml;



error_chain! {
    foreign_links {
        Alsa(alsa::Error);
        IO(std::io::Error);
        Toml(toml::de::Error);
    }
}




#[macro_export]
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
macro_rules! try_r {
    ($expr:expr, $ret:expr) => (match $expr {
        ::std::result::Result::Ok(val) => val,
        ::std::result::Result::Err(_) => {
            return $ret;
        },
    });
}



#[macro_export]
macro_rules! try_e {
    ($expr:expr) => {
        try_er!($expr, ())
    };
    ($expr:expr, $fmt:expr, $($arg:tt)+) => {
        try_er!($expr, (), $fmt, $(arg)+)
    };
    ($expr:expr, $fmt:expr) => {
        try_er!($expr, (), $fmt)
    }
}


#[macro_export]
macro_rules! try_er {
    ($expr:expr, $ret:expr) => (match $expr {
        ::std::result::Result::Ok(val) => val,
        ::std::result::Result::Err(err) => {
            error!("{:?}", err);
            ::std::process::exit(1);
        },
    });
    ($expr:expr, $ret:expr, $fmt:expr) => (match $expr {
        ::std::result::Result::Ok(val) => val,
        ::std::result::Result::Err(err) => {
            error!("Original error: {:?}", err);
            error!($fmt);
            std::process::exit(1);
        },
    });
    ($expr:expr, $ret:expr, $fmt:expr, $($arg:tt)+) => (match $expr {
        ::std::result::Result::Ok(val) => val,
        ::std::result::Result::Err(err) => {
            error!("Original error: {:?}", err);
            error!(format!($fmt, $(arg)+));
            std::process::exit(1);
        },
    })
}
