#![allow(unused, reason = "this file is a skeleton for the assignment")]

use crate::basic_types::PropagationStatusCP;
use crate::conjunction;
use crate::engine::cp::propagation::{
    PropagationContextMut, Propagator, PropagatorInitialisationContext,
};
use crate::predicates::PropositionalConjunction;
use crate::variables::IntegerVariable;
use crate::engine::cp::domain_events::DomainEvents;
use crate::engine::cp::propagation::propagation_context::ReadDomains;
use crate::branching::value_selection::PhaseSaving;
use crate::engine::constraint_satisfaction_solver;

/// A forward-checking propagator for a circuit constraint.
///
/// This propagator does not “solve” the full circuit constraint. Instead, it
/// focuses on propagation by filtering out candidate values that (if chosen)
/// would immediately lead to an incomplete cycle.
pub(crate) struct ForwardCheckingCircuitPropagator<Var> {
    successor: Box<[Var]>,
}

impl<Var> ForwardCheckingCircuitPropagator<Var> {
    pub(crate) fn new(successor: Box<[Var]>) -> Self {
        Self { successor }
    }
}

impl<Var: IntegerVariable + 'static> Propagator for ForwardCheckingCircuitPropagator<Var> {
    fn name(&self) -> &str {
        "ForwardCheckingCircuit"
    }

    fn propagate(&self, mut context: PropagationContextMut) -> PropagationStatusCP {
        let n = self.successor.len();

        // --- Case 1: For variables that are fixed ---
        // Follow the chain of fixed successors to check for cycles.
        // If a cycle is detected that does not cover all nodes, we return a failure.
        for i in 0..n {
            if context.is_fixed(&self.successor[i]) {
                let mut current = i;
                let mut visited = vec![false; n];
                while context.is_fixed(&self.successor[current]) {
                    if visited[current] {
                        // A cycle is detected. Count the nodes in the cycle.
                        let cycle_size = visited.iter().filter(|&&v| v).count();
                        if cycle_size < n {
                            // Incomplete cycle: propagation fails.
                            return Err(conjunction!().into());
                        }
                        break;
                    }
                    visited[current] = true;
                    // Subtract 1 to convert the domain value to a 0-index.
                    current = (context.lower_bound(&self.successor[current]) - 1) as usize;
                }
            }
        }
        // --- Case 2: For variables that are not yet fixed ---
        // For each unfixed variable, simulate each candidate.
        for i in 0..n {
            if !context.is_fixed(&self.successor[i]) {
                let lb = context.lower_bound(&self.successor[i]);
                let ub = context.upper_bound(&self.successor[i]);
                for candidate in lb..=ub {
                    // Simulate the effect of choosing `candidate` for variable i.
                    let candidate_closes = {
                        // Mark node i as visited.
                        let mut visited = vec![false; n];
                        visited[i] = true;
                        // Convert candidate value (1-indexed) to an index.
                        let mut current = (candidate - 1) as usize;
                        // Follow the chain using the lower bounds.
                        while current < n && !visited[current] {
                            visited[current] = true;
                            // Use the lower bound of successor[current] (converted to 0-index)
                            current = (context.lower_bound(&self.successor[current]) - 1) as usize;
                        }
                        // If we have looped back to i before visiting all nodes, then
                        // choosing `candidate` would close an incomplete cycle.
                        current == i && visited.iter().filter(|&&v| v).count() < n
                    };

                    if candidate_closes {
                        context.remove(&self.successor[i], candidate, conjunction!())?;
                    }
                }
            }
        }

        Ok(())
    }

    fn initialise_at_root(
        &mut self,
        init_context: &mut PropagatorInitialisationContext,
    ) -> Result<(), PropositionalConjunction> {
        let n = self.successor.len();
        for i in 0..n {
            init_context.register(self.successor[i].clone(), DomainEvents::ASSIGN);
        }
        Ok(())
    }
}
