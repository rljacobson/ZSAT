/*!

These are generic structures and algorithms: they do not depend on anything specific to this codebase or application.

*/

mod moving_average;
mod random;
mod true_false_vectors;
mod approximate_set;
mod statistics;
mod vector_pool;

pub use moving_average::{EMA, ExponentialMovingAverage};
pub use random::RandomGenerator;
pub use true_false_vectors::TFVectors;
pub use approximate_set::{ApproximateSet, OredIntegerSet};
pub use statistics::{Statistic, Statistics};
pub use vector_pool::*;

/*
/// Collection Literals
macro_rules! collection {
    // map-like
    ($($k:expr => $v:expr),* $(,)?) => {{
        use std::iter::{Iterator, IntoIterator};
        Iterator::collect(IntoIterator::into_iter([$(($k, $v),)*]))
    }};
    // set-like
    ($($v:expr),* $(,)?) => {{
        use std::iter::{Iterator, IntoIterator};
        Iterator::collect(IntoIterator::into_iter([$($v,)*]))
    }};
}
*/
