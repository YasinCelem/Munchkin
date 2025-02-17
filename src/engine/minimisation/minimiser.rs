use super::MinimisationContext;
use crate::engine::conflict_analysis::LearnedClause;
use crate::engine::cp::propagation::PropagationContext;

/// A trait which determines the behaviour of minimisers
pub(crate) trait Minimiser: Default {
    /// Takes as input a [`LearnedClause`] and minimises the clause based on some strategy.
    fn minimise(&mut self, context: MinimisationContext, learned_clause: &mut LearnedClause);
}

/// Recomputes the invariants of the [`LearnedClause`].
pub(crate) fn recompute_invariants(
    _context: PropagationContext,
    _learned_clause: &mut LearnedClause,
) {
    todo!()
}
