use super::MinimisationContext;
use super::Minimiser;
use crate::engine::conflict_analysis::LearnedClause;

pub(crate) struct SemanticMinimiser {
    // TODO
}

impl Default for SemanticMinimiser {
    #[allow(clippy::derivable_impls, reason = "Will be implemented")]
    fn default() -> Self {
        Self {}
    }
}

impl Minimiser for SemanticMinimiser {
    fn minimise(&mut self, _context: MinimisationContext, _learned_clause: &mut LearnedClause) {
        todo!()
    }
}
