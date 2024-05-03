pub trait UnwrapUnreachable<T> {
    fn unwrap_unreachable(self, #[cfg(debug_assertions)] msg: &str) -> T;
}

impl<T, E> UnwrapUnreachable<T> for std::result::Result<T, E>
where
    E: std::fmt::Debug,
{
    fn unwrap_unreachable(self, msg: &str) -> T {
        match self {
            Ok(t) => t,
            #[cfg(debug_assertions)]
            Err(e) => unreachable!(
                "Module:{}, Line:{}, Error:{:?} {msg}",
                module_path!(),
                line!(),
                e
            ),
            #[cfg(not(debug_assertions))]
            Err(_) => unreachable!(),
        }
    }
}

impl<T> UnwrapUnreachable<T> for std::option::Option<T> {
    fn unwrap_unreachable(self, msg: &str) -> T {
        match self {
            Some(t) => t,
            #[cfg(debug_assertions)]
            None => unreachable!("Module:{}, Line:{}, {msg}", module_path!(), line!()),
            #[cfg(not(debug_assertions))]
            None => unreachable!(),
        }
    }
}
