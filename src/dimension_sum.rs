use std::{fmt::Display, str::FromStr};

use nom::{
    character::complete::{char as char_parser, u32 as u32_parser},
    combinator::opt,
    multi::separated_list1,
    IResult,
};

use crate::{
    divides::{Divides, DoesNotDivide},
    impl_from_str_for_nom_parsable,
    nom_parsable::NomParsable,
};

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

impl IndeterminateDimensionSum {
    fn count_unknowns(&self) -> usize {
        self.addends.iter().filter(|o| o.is_none()).count()
    }

    fn sum_knowns(&self) -> u32 {
        self.addends.iter().flatten().sum()
    }

    pub fn infer_scale(&self, length: u32) -> Result<Option<u32>, DoesNotDivide<u32>> {
        if self.count_unknowns() == 0 {
            let scale = (Divides(length) / (Divides(self.sum_knowns())))?.0;

            Ok(Some(scale))
        } else {
            Ok(None)
        }
    }

    pub fn evaluate(self, length: u32, scale: u32) -> Result<DimensionSum, DoesNotDivide<u32>> {
        let unknown_count = self.count_unknowns();

        let addends = if unknown_count != 0 {
            let total = (Divides(length) / Divides(scale))?.0;

            let total_unknown = total - self.sum_knowns();

            let solution = (Divides(total_unknown) / Divides(self.count_unknowns() as u32))?.0;

            self.addends
                .into_iter()
                .map(|maybe_addend| maybe_addend.unwrap_or(solution))
                .collect()
        } else {
            self.addends.into_iter().flatten().collect()
        };

        Ok(DimensionSum { addends })
    }
}

impl NomParsable for IndeterminateDimensionSum {
    fn parser(input: &str) -> IResult<&str, IndeterminateDimensionSum> {
        let (input, values) = separated_list1(char_parser('+'), opt(u32_parser))(input)?;

        Ok((input, IndeterminateDimensionSum { addends: values }))
    }
}

impl FromStr for IndeterminateDimensionSum {
    impl_from_str_for_nom_parsable!();
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
