use super::ConflictAnalysisContext;
use crate::variables::Literal;

pub(crate) trait ConflictResolver {
    /// Resolves the current conflict.
    ///
    /// If the [`ConflictResolver`] learns a clause then it should be returned (and [`None`]
    /// otherwrise).
    fn resolve_conflict(&mut self, context: &mut ConflictAnalysisContext) -> Option<LearnedClause>;

    /// After creating the learned clause in [`ConflictResolver::resolve_conflict`], this method
    /// should put the solver in the "correct" state (e.g. by backtracking using
    /// [`ConflictAnalysisContext::backtrack`]).
    fn process(
        &mut self,
        learned_clause: Option<LearnedClause>,
        context: &mut ConflictAnalysisContext,
    ) -> Result<(), ()>;
}

/// A structure which stores a learned clause
///
/// There are two assumptions:
/// - The asserting literal (i.e. the literal of the current decision level) is placed at the `0`th
///   index of [`LearnedClause::literals`].
/// - A literal from the second-highest decision level is placed at the `1`st index of
///   [`LearnedClause::literals`].
///
/// A [`LearnedClause`] can be created using either [`LearnedClause::new`] or, in the case of a
/// unit learned clause, using [`LearnedClause::unit_learned_clause`].
#[derive(Clone, Debug)]
pub(crate) struct LearnedClause {
    pub(crate) literals: Vec<Literal>,
    pub(crate) backjump_level: usize,
}

#[allow(unused, reason = "will be used in the assignments")]
impl LearnedClause {
    pub(crate) fn new(literals: impl IntoIterator<Item = Literal>, backjump_level: usize) -> Self {
        Self {
            literals: literals.into_iter().collect::<Vec<_>>(),
            backjump_level,
        }
    }

    pub(crate) fn unit_learned_clause(literal: Literal) -> Self {
        Self {
            literals: vec![literal],
            backjump_level: 0,
        }
    }
}
