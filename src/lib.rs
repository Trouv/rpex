mod dimension_sum;
mod nom_parsable;
mod parser_combinators;
mod ratio_ext;
mod rectangle;
pub use rectangle::HyperRectangle;
mod sums_in_ratio;
pub use sums_in_ratio::{Partition, SumsInRatioEvaluationError};

pub type Rpex<const D: usize> = sums_in_ratio::IndeterminateSumsInRatio<D>;
