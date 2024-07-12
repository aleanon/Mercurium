use std::fmt::Display;

pub trait UnwrapUnreachable<T> {
    /// Marks the failure case as unreachable to help the compiler optimize the path away.
    /// Takes a &str that is only printed in debug build.
    fn unwrap_unreachable(self, msg: impl Display) -> T;
}

impl<T, E> UnwrapUnreachable<T> for std::result::Result<T, E>
where
    E: std::fmt::Debug,
{
    #[cfg(debug_assertions)]
    fn unwrap_unreachable(self, msg: impl Display) -> T {
        match self {
            Ok(t) => t,
            Err(e) => unreachable!("{msg}, error: {:?}", e),
        }
    }

    #[cfg(not(debug_assertions))]
    fn unwrap_unreachable(self, _: impl Display) -> T {
        match self {
            Ok(t) => t,
            Err(_) => unreachable!(),
        }
    }
}

impl<T> UnwrapUnreachable<T> for std::option::Option<T> {
    #[cfg(debug_assertions)]
    fn unwrap_unreachable(self, msg: impl Display) -> T {
        match self {
            Some(t) => t,
            None => unreachable!("{msg}"),
        }
    }

    #[cfg(not(debug_assertions))]
    fn unwrap_unreachable(self, _: impl Display) -> T {
        match self {
            Some(t) => t,
            None => unreachable!(),
        }
    }
}

#[macro_export]
/// returns a &str in debug build that consists of the module path, line number
/// and the passed in &str if any, in release build it returns an empty &str
macro_rules! debug_info {
    ($msg:expr) => {
        if cfg!(debug_assertions) {
            concat!("Module:", module_path!(), ", line:", line!(), ", ", $msg)
        } else {
            ""
        }
    };
    () => {
        if cfg!(debug_assertions) {
            concat!("Module:", module_path!(), ", line:", line!())
        } else {
            ""
        }
    };
}
