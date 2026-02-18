use crate::lcg::lcg::LCG;
use crate::lcg::rand::Rand;

/// A filtered skip in the random call sequence.
/// Stores the combined LCG at this skip position and a predicate to test state.
pub struct FilteredSkip {
    pub skip_lcg: LCG,
    pub filter: Box<dyn Fn(&mut Rand) -> bool + Send + Sync>,
}

impl FilteredSkip {
    pub fn new(current_index: i64, filter: Box<dyn Fn(&mut Rand) -> bool + Send + Sync>) -> Self {
        FilteredSkip {
            skip_lcg: LCG::JAVA.combine(current_index),
            filter,
        }
    }

    /// Check whether the given rand passes the filter after advancing by skip_lcg.
    pub fn check_state(&self, rand: &mut Rand) -> bool {
        rand.advance_lcg(&self.skip_lcg);
        (self.filter)(rand)
    }
}
