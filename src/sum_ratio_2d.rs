use std::str::FromStr;

use nom::{
    character::complete::char as char_parser, combinator::all_consuming, error::Error,
    sequence::separated_pair, Finish, IResult,
};

use crate::{
    dimension_sum::{DimensionSum, DoesNotDivide, IndeterminateDimensionSum},
    rectangle::HyperRectangle,
};
use thiserror::Error;

struct SumRatio2d {
    x_sum: DimensionSum,
    y_sum: DimensionSum,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct IndeterminateSumRatio2d {
    pub x_sum: IndeterminateDimensionSum,
    pub y_sum: IndeterminateDimensionSum,
}

#[derive(Error, Debug)]
enum SumRatio2dEvaluationError {
    #[error("evaluated x_scale {x_scale} and y_scale {y_scale} are unequal")]
    UnequalScales { x_scale: u32, y_scale: u32 },
    #[error("division error occurred: {0}")]
    DoesNotDivide(#[from] DoesNotDivide),
}

impl IndeterminateSumRatio2d {
    pub fn parser(input: &str) -> IResult<&str, IndeterminateSumRatio2d> {
        let (input, (x, y)) = separated_pair(
            IndeterminateDimensionSum::parser,
            char_parser(':'),
            IndeterminateDimensionSum::parser,
        )(input)?;

        Ok((input, IndeterminateSumRatio2d { x_sum: x, y_sum: y }))
    }

    fn evaluate(
        self,
        rectangle: HyperRectangle<2>,
    ) -> Result<(SumRatio2d, u32), SumRatio2dEvaluationError> {
        let maybe_x_scale = self.x_sum.infer_scale(rectangle.lengths[0])?;
        let maybe_y_scale = self.y_sum.infer_scale(rectangle.lengths[1])?;

        let scale = match (maybe_x_scale, maybe_y_scale) {
            (Some(x_scale), Some(y_scale)) if x_scale == y_scale => x_scale,
            (Some(x_scale), Some(y_scale)) => {
                Err(SumRatio2dEvaluationError::UnequalScales { x_scale, y_scale })?
            }
            (Some(scale), None) | (None, Some(scale)) => scale,
            (None, None) => 1,
        };

        let x_sum = self.x_sum.evaluate(rectangle.lengths[0], scale)?;
        let y_sum = self.y_sum.evaluate(rectangle.lengths[1], scale)?;

        Ok((SumRatio2d { x_sum, y_sum }, scale))
    }
}

impl FromStr for IndeterminateSumRatio2d {
    type Err = Error<String>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (_, dim_sum) = all_consuming(IndeterminateSumRatio2d::parser)(s)
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
            IndeterminateSumRatio2d::from_str("+:").unwrap(),
            IndeterminateSumRatio2d {
                x_sum: IndeterminateDimensionSum {
                    addends: vec![None, None]
                },
                y_sum: IndeterminateDimensionSum {
                    addends: vec![None]
                }
            }
        );
        assert_eq!(
            IndeterminateSumRatio2d::from_str("+:++").unwrap(),
            IndeterminateSumRatio2d {
                x_sum: IndeterminateDimensionSum {
                    addends: vec![None, None]
                },
                y_sum: IndeterminateDimensionSum {
                    addends: vec![None, None, None]
                }
            }
        );
    }

    #[test]
    fn we_can_parse_ratio_with_values() {
        assert_eq!(
            IndeterminateSumRatio2d::from_str("1+2:3").unwrap(),
            IndeterminateSumRatio2d {
                x_sum: IndeterminateDimensionSum {
                    addends: vec![Some(1), Some(2)]
                },
                y_sum: IndeterminateDimensionSum {
                    addends: vec![Some(3)]
                }
            }
        );
        assert_eq!(
            IndeterminateSumRatio2d::from_str("12+34:56++789").unwrap(),
            IndeterminateSumRatio2d {
                x_sum: IndeterminateDimensionSum {
                    addends: vec![Some(12), Some(34)]
                },
                y_sum: IndeterminateDimensionSum {
                    addends: vec![Some(56), None, Some(789)]
                }
            }
        );
    }

    #[test]
    fn we_cannot_parse_ratio_with_bad_dim_sum() {
        assert!(IndeterminateSumRatio2d::from_str("1++1x:1").is_err());
        assert!(IndeterminateSumRatio2d::from_str("1:1+x").is_err());
    }

    #[test]
    fn we_cannot_parse_ratio_with_bad_separator() {
        assert!(IndeterminateSumRatio2d::from_str("1+1").is_err());
        assert!(IndeterminateSumRatio2d::from_str("1+1-1+1").is_err());
    }

    #[test]
    fn we_cannot_parse_ratio_with_extra_characters() {
        assert!(IndeterminateSumRatio2d::from_str("1+1::").is_err());
        assert!(IndeterminateSumRatio2d::from_str("x1+1:1+1").is_err());
    }
}
