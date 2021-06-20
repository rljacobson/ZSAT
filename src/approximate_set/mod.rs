/*!
An `ApproximateSet` has the properties:
  1. Membership is not guaranteed to be correct but is "approximately" correct.
  2. Non-membership is guaranteed to be correct.
  3. Querying is very fast.
  4. Space efficient.
*/

mod approximate_set_trait;
mod ored_integer_set;

pub use approximate_set_trait::ApproximateSet;
pub use ored_integer_set::OredIntegerSet;