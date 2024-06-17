use nom::IResult;

pub trait NomParsable {
    fn parser(input: &str) -> IResult<&str, Self>
    where
        Self: Sized;
}

#[macro_export]
macro_rules! impl_from_str_for_nom_parsable {
    () => {
        type Err = nom::error::Error<String>;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            let (_, result) = <_ as nom::Finish<_, _, _>>::finish(nom::combinator::all_consuming(
                <Self as crate::nom_parsable::NomParsable>::parser,
            )(s))
            .map_err(|nom::error::Error { input, code }| nom::error::Error {
                input: input.to_string(),
                code,
            })?;

            Ok(result)
        }
    };
}
