#![feature(backtrace)]

use std::backtrace::Backtrace;
use std::convert::TryFrom;
use std::fmt::{Debug, Display, Formatter};
use std::ops::{Deref, DerefMut};

#[derive(Copy, Clone, Default, Debug, Eq, PartialEq)]
pub struct Id(u32);

#[derive(Debug)]
pub struct AnyError {
    inner: Box<dyn std::error::Error>,
    back_trace: Backtrace,
}

pub type AnyErrorResult<T> = Result<T, AnyError>;

impl Id {
    pub fn as_usize(&self) -> usize {
        self.0 as usize
    }
}

impl Deref for Id {
    type Target = u32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Id {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<u32> for Id {
    fn from(id: u32) -> Self {
        Self(id)
    }
}

impl From<usize> for Id {
    fn from(id: usize) -> Self {
        Self(u32::try_from(id).unwrap())
    }
}

impl Display for Id {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Display for AnyError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}\nstack backtrace from AnyError:\n{}",
            self.inner, self.back_trace
        )
    }
}

impl<E: 'static + std::error::Error> From<E> for AnyError {
    fn from(e: E) -> Self {
        Self {
            inner: e.into(),
            back_trace: Backtrace::capture(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;
    use std::fmt::{Display, Formatter};

    use crate::AnyError;

    #[derive(Debug)]
    struct TestError {}

    impl Display for TestError {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(f, "TestError.display()")
        }
    }

    impl Error for TestError {}

    #[test]
    fn display_any_error() {
        let any_error: AnyError = TestError {}.into();
        println!("{}", any_error);
        println!("{:?}", any_error);
    }
}
