use std::backtrace::Backtrace;
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub struct AnyError {
    inner: Box<dyn std::error::Error>,
    back_trace: Backtrace,
}

pub type AnyErrorResult<T> = Result<T, AnyError>;

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

    use crate::error::AnyError;

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
