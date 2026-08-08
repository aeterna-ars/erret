use std::fmt;

pub struct ErrRet {
    inner: Box<dyn std::error::Error + Send + Sync + 'static>,
}

impl ErrRet {
    pub fn new<E>(err: E) -> Self
    where
        E: std::error::Error + Send + Sync + 'static,
    {
        ErrRet { inner: Box::new(err) }
    }
}

impl fmt::Display for ErrRet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.inner)
    }
}

impl fmt::Debug for ErrRet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.inner)
    }
}

impl<E> From<E> for ErrRet
where
    E: std::error::Error + Send + Sync + 'static,
{
    fn from(err: E) -> Self {
        ErrRet::new(err)
    }
}

pub type ErrResult<T, E = ErrRet> = std::result::Result<T, E>;