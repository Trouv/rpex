use std::fmt::Display;

use fraction::{Integer, Ratio};
use thiserror::Error;

#[derive(Debug, Error)]
#[error("ratio not an integer {0}")]
pub struct NotAnInteger<T: Display>(Ratio<T>);

pub trait RatioExt<T: Clone>
where
    T: Clone + Integer + Display,
{
    fn try_to_integer(self) -> Result<T, NotAnInteger<T>>;
}

impl<T> RatioExt<T> for Ratio<T>
where
    T: Clone + Integer + Display,
{
    fn try_to_integer(self) -> Result<T, NotAnInteger<T>> {
        if self.is_integer() {
            Ok(self.to_integer())
        } else {
            Err(NotAnInteger(self))
        }
    }
}
