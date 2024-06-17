use std::str::FromStr;

use nom::{
    character::complete::{char as char_parser, u32 as u32_parser},
    combinator::all_consuming,
    error::Error,
    sequence::separated_pair,
    Finish, IResult,
};

struct Rectangle {
    width: u32,
    height: u32,
}

impl Rectangle {
    fn parser(input: &str) -> IResult<&str, Rectangle> {
        let (input, (width, height)) =
            separated_pair(u32_parser, char_parser('x'), u32_parser)(input)?;

        Ok((input, Rectangle { width, height }))
    }
}

impl FromStr for Rectangle {
    type Err = Error<String>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (_, resolution) =
            all_consuming(Rectangle::parser)(s)
                .finish()
                .map_err(|Error { input, code }| Error {
                    input: input.to_string(),
                    code,
                })?;

        Ok(resolution)
    }
}
