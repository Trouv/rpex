use std::str::FromStr;

use nom::{
    character::complete::{char as char_parser, u32 as u32_parser},
    IResult,
};

use crate::{
    impl_from_str_for_nom_parsable, nom_parsable::NomParsable,
    parser_combinators::separated_list_m_n,
};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct HyperRectangle<const D: usize> {
    pub lengths: [u32; D],
}

impl<const D: usize> NomParsable for HyperRectangle<D> {
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
    impl_from_str_for_nom_parsable!();
}
