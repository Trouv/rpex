use std::str::FromStr;

use nom::{
    character::complete::{char as char_parser, u32 as u32_parser},
    combinator::all_consuming,
    error::Error,
    Finish, IResult,
};

use crate::parser_combinators::separated_list_m_n;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct HyperRectangle<const D: usize> {
    pub lengths: [u32; D],
}

impl<const D: usize> HyperRectangle<D> {
    fn parser(input: &str) -> IResult<&str, HyperRectangle<D>> {
        assert!(D != 0, "0-dimensional HyperRectangles not supported");

        let (input, lengths) = separated_list_m_n(D, D, char_parser('x'), u32_parser)(input)?;

        Ok((
            input,
            HyperRectangle {
                lengths: lengths
                    .try_into()
                    .expect("we parsed lengths to have D elements"),
            },
        ))
    }
}

impl<const D: usize> FromStr for HyperRectangle<D> {
    type Err = Error<String>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (_, resolution) = all_consuming(HyperRectangle::<D>::parser)(s)
            .finish()
            .map_err(|Error { input, code }| Error {
                input: input.to_string(),
                code,
            })?;

        Ok(resolution)
    }
}
