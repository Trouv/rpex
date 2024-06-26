use std::{collections::HashSet, str::FromStr};

use fraction::Integer;
use itertools::Itertools;
use nom::{character::complete::char as char_parser, IResult};

use crate::{
    dimension_sum::{
        AddendWithOffset, DimensionSum, DimensionSumEvaluationError, IndeterminateDimensionSum,
    },
    impl_from_str_for_nom_parsable,
    nom_parsable::NomParsable,
    parser_combinators::separated_list_m_n,
    ratio_ext::NotAnInteger,
    rectangle::HyperRectangle,
};
use thiserror::Error;

pub struct SumsInRatio<const D: usize> {
    sums: [DimensionSum; D],
}

pub struct Partition<'a, const D: usize> {
    pub ratio_position: [u32; D],
    pub ratio: [&'a u32; D],
}

impl<const D: usize> SumsInRatio<D> {
    pub fn iter_partitions(&self) -> impl Iterator<Item = Partition<D>> {
        self.sums
            .iter()
            .map(|dim_sum| dim_sum.iter_with_offsets().collect::<Vec<_>>())
            .multi_cartesian_product()
            .map(|dimension_sums_with_offsets| {
                let (addends, offsets): (Vec<&u32>, Vec<u32>) = dimension_sums_with_offsets
                    .into_iter()
                    .map(|AddendWithOffset { addend, offset }| (addend, offset))
                    .unzip();

                Partition {
                    ratio_position: offsets.try_into().expect(""),
                    ratio: addends.try_into().expect(""),
                }
            })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IndeterminateSumsInRatio<const D: usize> {
    pub sums: [IndeterminateDimensionSum; D],
}

#[derive(Error, Debug)]
pub enum SumsInRatioEvaluationError {
    #[error("inferred scales from dimensions are unequal: {0:?}")]
    UnequalScales(HashSet<u32>),
    #[error("division error occurred: {0}")]
    DoesNotDivide(#[from] NotAnInteger<u32>),
    #[error("unable to evaluate dimension sum: {0}")]
    DimensionSumEvaluation(#[from] DimensionSumEvaluationError),
}

impl<const D: usize> IndeterminateSumsInRatio<D> {
    pub fn evaluate(
        self,
        rectangle: HyperRectangle<D>,
    ) -> Result<(SumsInRatio<D>, u32), SumsInRatioEvaluationError> {
        let inferred_scales = self
            .sums
            .iter()
            .zip(rectangle.lengths)
            .flat_map(|(sum, length)| sum.infer_scale(length).transpose())
            .collect::<Result<HashSet<_>, _>>()?;

        let known_scale = match inferred_scales.len() {
            0 => 1,
            1 => inferred_scales
                .into_iter()
                .last()
                .expect("inferred_scales has length 1"),
            _ => return Err(SumsInRatioEvaluationError::UnequalScales(inferred_scales)),
        };

        let scale = rectangle
            .lengths
            .iter()
            .fold(known_scale, |gcd, length| gcd.gcd(length));

        let scale_factor = known_scale / scale;

        let evaluated_sums = self
            .sums
            .map(|dim_sum| dim_sum * scale_factor)
            .into_iter()
            .zip(rectangle.lengths)
            .map(|(sum, length)| sum.evaluate(length / scale))
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

impl<const D: usize> NomParsable for IndeterminateSumsInRatio<D> {
    fn parser(input: &str) -> IResult<&str, IndeterminateSumsInRatio<D>> {
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
}

impl<const D: usize> FromStr for IndeterminateSumsInRatio<D> {
    impl_from_str_for_nom_parsable!();
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
