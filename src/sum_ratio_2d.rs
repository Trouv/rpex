use std::{collections::HashSet, str::FromStr};

use nom::{
    character::complete::char as char_parser, combinator::all_consuming, error::Error, Finish,
    IResult,
};

use crate::{
    dimension_sum::{DimensionSum, DoesNotDivide, IndeterminateDimensionSum},
    parser_combinators::separated_list_m_n,
    rectangle::HyperRectangle,
};
use thiserror::Error;

struct SumsInRatio<const D: usize> {
    sums: [DimensionSum; D],
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct IndeterminateSumsInRatio<const D: usize> {
    pub sums: [IndeterminateDimensionSum; D],
}

#[derive(Error, Debug)]
enum SumsInRatioEvaluationError {
    #[error("inferred scales from dimensions are unequal: {0:?}")]
    UnequalScales(HashSet<u32>),
    #[error("division error occurred: {0}")]
    DoesNotDivide(#[from] DoesNotDivide),
}

impl<const D: usize> IndeterminateSumsInRatio<D> {
    pub fn parser(input: &str) -> IResult<&str, IndeterminateSumsInRatio<D>> {
        assert!(D != 0, "0-dimensional SumsInRatio are not supported");

        let (input, sums) =
            separated_list_m_n(D, D, char_parser(':'), IndeterminateDimensionSum::parser)(input)?;

        Ok((
            input,
            IndeterminateSumsInRatio {
                sums: sums.try_into().expect("we parsed sums to have D elements"),
            },
        ))
    }

    fn evaluate(
        self,
        rectangle: HyperRectangle<D>,
    ) -> Result<(SumsInRatio<D>, u32), SumsInRatioEvaluationError> {
        let inferred_scales = self
            .sums
            .iter()
            .zip(rectangle.lengths)
            .flat_map(|(sum, length)| sum.infer_scale(length).transpose())
            .collect::<Result<HashSet<_>, _>>()?;

        let scale = match inferred_scales.len() {
            0 => 1,
            1 => inferred_scales
                .into_iter()
                .last()
                .expect("inferred_scales has length 1"),
            _ => return Err(SumsInRatioEvaluationError::UnequalScales(inferred_scales)),
        };

        let evaluated_sums = self
            .sums
            .into_iter()
            .zip(rectangle.lengths)
            .map(|(sum, length)| sum.evaluate(length, scale))
            .collect::<Result<Vec<_>, _>>()?;

        Ok((
            SumsInRatio {
                sums: evaluated_sums
                    .try_into()
                    .expect("sums is built from arrays of length D"),
            },
            scale,
        ))
    }
}

impl<const D: usize> FromStr for IndeterminateSumsInRatio<D> {
    type Err = Error<String>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (_, dim_sum) = all_consuming(IndeterminateSumsInRatio::parser)(s)
            .finish()
            .map_err(|Error { input, code }| Error {
                input: input.to_string(),
                code,
            })?;

        Ok(dim_sum)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn we_can_parse_ratio_with_no_values() {
        assert_eq!(
            IndeterminateSumsInRatio::from_str("+:").unwrap(),
            IndeterminateSumsInRatio {
                sums: [
                    IndeterminateDimensionSum {
                        addends: vec![None, None]
                    },
                    IndeterminateDimensionSum {
                        addends: vec![None]
                    }
                ]
            }
        );
        assert_eq!(
            IndeterminateSumsInRatio::from_str("+:++").unwrap(),
            IndeterminateSumsInRatio {
                sums: [
                    IndeterminateDimensionSum {
                        addends: vec![None, None]
                    },
                    IndeterminateDimensionSum {
                        addends: vec![None, None, None]
                    }
                ]
            }
        );
    }

    #[test]
    fn we_can_parse_ratio_with_values() {
        assert_eq!(
            IndeterminateSumsInRatio::from_str("1+2:3").unwrap(),
            IndeterminateSumsInRatio {
                sums: [
                    IndeterminateDimensionSum {
                        addends: vec![Some(1), Some(2)]
                    },
                    IndeterminateDimensionSum {
                        addends: vec![Some(3)]
                    }
                ]
            }
        );
        assert_eq!(
            IndeterminateSumsInRatio::from_str("12+34:56++789").unwrap(),
            IndeterminateSumsInRatio {
                sums: [
                    IndeterminateDimensionSum {
                        addends: vec![Some(12), Some(34)]
                    },
                    IndeterminateDimensionSum {
                        addends: vec![Some(56), None, Some(789)]
                    }
                ]
            }
        );
    }

    #[test]
    fn we_cannot_parse_ratio_with_bad_dim_sum() {
        assert!(IndeterminateSumsInRatio::<2>::from_str("1++1x:1").is_err());
        assert!(IndeterminateSumsInRatio::<2>::from_str("1:1+x").is_err());
    }

    #[test]
    fn we_cannot_parse_ratio_with_bad_separator() {
        assert!(IndeterminateSumsInRatio::<2>::from_str("1+1").is_err());
        assert!(IndeterminateSumsInRatio::<2>::from_str("1+1-1+1").is_err());
    }

    #[test]
    fn we_cannot_parse_ratio_with_extra_characters() {
        assert!(IndeterminateSumsInRatio::<2>::from_str("1+1::").is_err());
        assert!(IndeterminateSumsInRatio::<2>::from_str("x1+1:1+1").is_err());
    }
}
