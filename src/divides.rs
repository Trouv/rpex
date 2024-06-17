use std::{
    fmt::Display,
    ops::{Div, Rem},
};

use num_traits::Zero;
use thiserror::Error;

pub struct Divides<T>(pub T)
where
    T: Zero + Div<Output = T> + Display + PartialEq,
    for<'a> &'a T: Rem<Output = T>;

#[derive(Debug, Error)]
#[error("{0} does not divide {1}")]
pub struct DoesNotDivide<T: Display>(T, T);

impl<T> Div for Divides<T>
where
    T: Zero + Div<Output = T> + Display + PartialEq,
    for<'a> &'a T: Rem<Output = T>,
{
    type Output = Result<Divides<T>, DoesNotDivide<T>>;

    fn div(self, rhs: Self) -> Self::Output {
        if &self.0 % &rhs.0 == T::zero() {
            Ok(Divides(self.0 / rhs.0))
        } else {
            Err(DoesNotDivide(self.0, rhs.0))
        }
    }
}
