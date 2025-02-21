#![allow(unused, reason = "this file is a skeleton for the assignment")]

use crate::basic_types::PropagationStatusCP;
use crate::engine::cp::propagation::PropagationContextMut;
use crate::engine::cp::propagation::Propagator;
use crate::engine::cp::propagation::PropagatorInitialisationContext;
use crate::predicates::PropositionalConjunction;
use crate::variables::IntegerVariable;

/// Propagator for constraint `element([x_1, \ldots, x_n], i, e)`, where `x_j` are
///  variables, `i` is an integer variable, and `e` is a variable, which holds iff `x_i = e`
///
/// Note that this propagator is 1-indexed
pub(crate) struct ElementPropagator<IndexVar, ArrayVar, RhsVar> {
    index: IndexVar,
    array: Box<[ArrayVar]>,
    rhs: RhsVar,
    // TODO: you can add more fields here!
}

impl<IndexVar, ArrayVar, RhsVar> ElementPropagator<IndexVar, ArrayVar, RhsVar> {
    pub(crate) fn new(index: IndexVar, array: Box<[ArrayVar]>, rhs: RhsVar) -> Self {
        ElementPropagator { index, array, rhs }
    }
}

impl<IndexVar, ArrayVar, RhsVar> Propagator for ElementPropagator<IndexVar, ArrayVar, RhsVar>
where
    IndexVar: IntegerVariable + 'static,
    ArrayVar: IntegerVariable + 'static,
    RhsVar: IntegerVariable + 'static,
{
    fn name(&self) -> &str {
        "Element"
    }

    fn propagate(&self, _context: PropagationContextMut) -> PropagationStatusCP {
        todo!()
    }

    fn initialise_at_root(
        &mut self,
        _context: &mut PropagatorInitialisationContext,
    ) -> Result<(), PropositionalConjunction> {
        todo!()
    }
}
