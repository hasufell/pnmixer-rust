use alsa;
use std::convert::From;
use std;



error_chain! {
    foreign_links {
        Alsa(alsa::Error);
    }

}


pub trait CHErr {
    type Item;
    fn cherr(self) -> Result<Self::Item>;
}


impl<A, E: std::error::Error> CHErr for std::result::Result<A, E>
    where Error: std::convert::From<E>
{
    type Item = A;
    fn cherr(self) -> Result<Self::Item> {
        return self.map_err(From::from);
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
        std::result::Result::Ok(val) => val,
        std::result::Result::Err(err) => {
            warn!("Original error: {:?}", err);
            warn!($fmt);
            return $ret;
        },
    });
    ($expr:expr, $ret:expr, $fmt:expr, $($arg:tt)+) => (match $expr {
        std::result::Result::Ok(val) => val,
        std::result::Result::Err(err) => {
            warn!("Original error: {:?}", err);
            warn!(format!($fmt, $(arg)+));
            return $ret;
        },
    })
}


#[macro_export]
macro_rules! try_r {
    ($expr:expr, $ret:expr) => (match $expr {
        std::result::Result::Ok(val) => val,
        std::result::Result::Err(err) => {
            return $ret;
        },
    });
}
