use super::ConflictAnalysisContext;
use super::ConflictResolver;
use super::LearnedClause;
#[cfg(doc)]
use crate::engine::ConstraintSatisfactionSolver;

#[derive(Default, Debug)]
pub(crate) struct AllDecisionLearning {
    // TODO
}

impl ConflictResolver for AllDecisionLearning {
    /// Compute the clause using all-decision learning.
    ///
    /// The learned clause which is created by
    /// this method contains a single variable at the current decision level (stored at index 0
    /// of [`LearnedClause::literals`]); the variable with the second highest
    /// decision level is stored at index 1 in [`LearnedClause::literals`] and its
    /// decision level is (redundantly) stored in [`LearnedClause::backjump_level`], which
    /// is used when backtracking in ([`ConstraintSatisfactionSolver`]).
    ///
    /// See the utility methods in [`ConflictAnalysisContext`] to get a better overview of which
    /// functions are available to you.
    fn resolve_conflict(
        &mut self,
        _context: &mut ConflictAnalysisContext,
    ) -> Option<LearnedClause> {
        todo!()
    }

    fn process(
        &mut self,
        _learned_clause: Option<LearnedClause>,
        _context: &mut ConflictAnalysisContext,
    ) -> Result<(), ()> {
        todo!()
    }
}
