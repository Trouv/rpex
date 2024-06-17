use std::{fmt::Display, str::FromStr};

use nom::{
    character::complete::{char as char_parser, u32 as u32_parser},
    combinator::{all_consuming, opt},
    error::Error,
    multi::separated_list1,
    Finish, IResult,
};
use thiserror::Error;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct DimensionSum {
    addends: Vec<u32>,
}

impl DimensionSum {
    fn sum(&self) -> u32 {
        self.addends.iter().sum()
    }

    fn infer_scale(&self, length: u32) -> u32 {
        length / self.sum()
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct IndeterminateDimensionSum {
    pub addends: Vec<Option<u32>>,
}

#[derive(Debug, Error)]
#[error("{0} does not divide {1}")]
pub struct DoesNotDivide(u32, u32);

impl DoesNotDivide {
    fn try_divide(dividend: u32, divisor: u32) -> Result<u32, DoesNotDivide> {
        if dividend % divisor == 0 {
            Ok(dividend / divisor)
        } else {
            Err(DoesNotDivide(divisor, dividend))
        }
    }
}

impl IndeterminateDimensionSum {
    pub fn parser(input: &str) -> IResult<&str, IndeterminateDimensionSum> {
        let (input, values) = separated_list1(char_parser('+'), opt(u32_parser))(input)?;

        Ok((input, IndeterminateDimensionSum { addends: values }))
    }

    fn count_unknowns(&self) -> usize {
        self.addends.iter().filter(|o| o.is_none()).count()
    }

    fn sum_knowns(&self) -> u32 {
        self.addends.iter().flatten().sum()
    }

    pub fn infer_scale(&self, length: u32) -> Result<Option<u32>, DoesNotDivide> {
        if self.count_unknowns() == 0 {
            let scale = DoesNotDivide::try_divide(length, self.sum_knowns())?;

            Ok(Some(scale))
        } else {
            Ok(None)
        }
    }

    pub fn evaluate(self, length: u32, scale: u32) -> Result<DimensionSum, DoesNotDivide> {
        let total = DoesNotDivide::try_divide(length, scale)?;

        let total_unknown = total - self.sum_knowns();

        let solution = DoesNotDivide::try_divide(total_unknown, self.count_unknowns() as u32)?;

        let addends = self
            .addends
            .into_iter()
            .map(|maybe_addend| maybe_addend.unwrap_or(solution))
            .collect();

        Ok(DimensionSum { addends })
    }
}

impl FromStr for IndeterminateDimensionSum {
    type Err = Error<String>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (_, dim_sum) = all_consuming(IndeterminateDimensionSum::parser)(s)
            .finish()
            .map_err(|Error { input, code }| Error {
                input: input.to_string(),
                code,
            })?;

        Ok(dim_sum)
    }
}

impl Display for IndeterminateDimensionSum {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let string_representations = self
            .addends
            .iter()
            .map(|addend| match addend {
                Some(a) => a.to_string(),
                None => "".to_string(),
            })
            .collect::<Vec<_>>();

        let joined = string_representations.join("+");

        f.write_str(joined.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn we_can_parse_dim_sum_with_all_nones() {
        assert_eq!(
            IndeterminateDimensionSum::from_str("").unwrap(),
            IndeterminateDimensionSum {
                addends: vec![None]
            }
        );
        assert_eq!(
            IndeterminateDimensionSum::from_str("+").unwrap(),
            IndeterminateDimensionSum {
                addends: vec![None, None]
            }
        );
        assert_eq!(
            IndeterminateDimensionSum::from_str("+++").unwrap(),
            IndeterminateDimensionSum {
                addends: vec![None, None, None, None]
            }
        );
    }

    #[test]
    fn we_can_parse_dim_sum_with_some_nones() {
        assert_eq!(
            IndeterminateDimensionSum::from_str("1+").unwrap(),
            IndeterminateDimensionSum {
                addends: vec![Some(1), None]
            }
        );
        assert_eq!(
            IndeterminateDimensionSum::from_str("+2").unwrap(),
            IndeterminateDimensionSum {
                addends: vec![None, Some(2)]
            }
        );
        assert_eq!(
            IndeterminateDimensionSum::from_str("1+2++45+56").unwrap(),
            IndeterminateDimensionSum {
                addends: vec![Some(1), Some(2), None, Some(45), Some(56)]
            }
        );
    }

    #[test]
    fn we_can_parse_dim_sum_with_no_nones() {
        assert_eq!(
            IndeterminateDimensionSum::from_str("12").unwrap(),
            IndeterminateDimensionSum {
                addends: vec![Some(12)]
            }
        );
        assert_eq!(
            IndeterminateDimensionSum::from_str("1+23").unwrap(),
            IndeterminateDimensionSum {
                addends: vec![Some(1), Some(23)]
            }
        );
        assert_eq!(
            IndeterminateDimensionSum::from_str("12+3+4+56").unwrap(),
            IndeterminateDimensionSum {
                addends: vec![Some(12), Some(3), Some(4), Some(56)]
            }
        );
    }

    #[test]
    fn we_cannot_parse_dim_sum_with_extra_characters() {
        assert!(IndeterminateDimensionSum::from_str("1++1x").is_err());
        assert!(IndeterminateDimensionSum::from_str("1+x+1").is_err());
        assert!(IndeterminateDimensionSum::from_str("x1++1").is_err());
    }

    #[test]
    fn we_cannot_parse_dim_sum_with_bad_numbers() {
        assert!(IndeterminateDimensionSum::from_str("1+y").is_err());
        assert!(IndeterminateDimensionSum::from_str("x+2").is_err());
    }

    #[test]
    fn we_cannot_parse_dim_sum_with_bad_separator() {
        assert!(IndeterminateDimensionSum::from_str("1-1").is_err());
        assert!(IndeterminateDimensionSum::from_str("1+-2").is_err());
    }
}
