use nom::{error::ParseError, multi::many_m_n, sequence::pair, Err, IResult, InputLength, Parser};

pub fn separated_list_m_n<I, O, O2, E, F, G>(
    min: usize,
    max: usize,
    mut sep: G,
    mut f: F,
) -> impl FnMut(I) -> IResult<I, Vec<O>, E>
where
    I: Clone + InputLength,
    F: FnMut(I) -> IResult<I, O, E>,
    G: FnMut(I) -> IResult<I, O2, E>,
    E: ParseError<I>,
{
    move |input: I| {
        let (input, head) = match f(input.clone()) {
            Err(Err::Error(_)) if min == 0 => return Ok((input, vec![])),
            Err(e) => return Err(e),
            Ok(result) => result,
        };

        let (input, tail) = many_m_n(
            min - 1,
            max - 1,
            pair(&mut sep, &mut f).map(|(_separator, element)| element),
        )(input)?;

        let result = std::iter::once(head).chain(tail).collect::<Vec<_>>();

        Ok((input, result))
    }
}
