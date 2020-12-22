pub use anyhow;
pub type Result<T> = anyhow::Result<T>;
pub type Error = anyhow::Error;
pub use std::result::Result as StdResult;

#[inline]
pub fn raise<T, Error>(err: Error) -> self::Result<T>
where
    Error: std::error::Error + Send + Sync + 'static,
{
    Err(err.into())
}

#[macro_export]
macro_rules! simple_error {
    ($fmt:expr $(, $arg:tt)*) => {
        $crate::result::anyhow::anyhow!($fmt $(, $arg)*)
    };
}

#[cfg(test)]
mod test {
    use thiserror::Error;

    #[derive(Error, Debug)]
    enum Error {
        #[error("{0}")]
        Hoge(String),
    }

    #[cfg(test)]
    fn raise_anyerror() -> super::Result<()> {
        Err(Error::Hoge("hoge".into()).into())
    }

    #[test]
    fn test_raise_anyerror() {
        assert_eq!("hoge", format!("{}", raise_anyerror().unwrap_err()));
    }

    #[test]
    fn test_simple_error() {
        simple_error!("{} {}", "hoge", "moge");
    }
}
