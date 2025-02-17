use super::MinimisationContext;
use super::Minimiser;
use crate::engine::conflict_analysis::LearnedClause;

pub(crate) struct SemanticMinimiser {
    // TODO
}

impl Default for SemanticMinimiser {
    fn default() -> Self {
        todo!()
    }
}

impl Minimiser for SemanticMinimiser {
    fn minimise(&mut self, _context: MinimisationContext, _learned_clause: &mut LearnedClause) {
        todo!()
    }
}
