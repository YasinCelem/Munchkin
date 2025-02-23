#![allow(unused, reason = "this file is a skeleton for the assignment")]

use crate::basic_types::{HashSet, PropagationStatusCP};
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
        for i in 0..n {
            if context.is_fixed(&self.successor[i]) {
                let mut current = i;
                let mut chain: Vec<usize> = Vec::new();
                let mut visited = HashSet::new();
    
                chain.push(current);
                let _=visited.insert(current);
    
                // Follow the chain, converting the fixed 1-based value to a 0-based index.
                while context.is_fixed(&self.successor[current]) {
                    let v = context.lower_bound(&self.successor[current]); // v is 1-based
                    let next = (v - 1) as usize; // convert to 0-based index
                    if visited.contains(&next) {
                        // If the cycle is incomplete, signal a conflict immediately.
                        if chain.len() < n {
                            // For example, force a domain wipe-out by tightening bounds:
                            context.set_lower_bound(&self.successor[i], 0, conjunction!())?;
                            context.set_upper_bound(&self.successor[i], -1, conjunction!())?;
                            return Ok(());
                        }
                        break;
                    }
                    chain.push(next);
                    let _=visited.insert(next);
                    current = next;
                }
                // If the chain is incomplete, remove the "back edge" that would close the cycle.
                if chain.len() < n {
                    // Remove the edge from the last node in the chain to the first node.
                    let first = chain[0] as i32 + 1; // forbidden value is (chain[0] + 1)
                    let last = chain[chain.len() - 1];
                    context.remove(&self.successor[last], first, conjunction!())?;
                }
                // Also, prevent self-loops: for any node i, remove i+1 from its domain.
                let last_index = chain[chain.len() - 1];
                if context.contains(&self.successor[last_index], (last_index as i32) + 1) {
                    context.remove(&self.successor[last_index], (last_index as i32) + 1, conjunction!())?;
                }
            }
        }
        Ok(())
    }
    

    fn initialise_at_root(
        &mut self,
        context: &mut PropagatorInitialisationContext,
    ) -> Result<(), PropositionalConjunction> {
        
        for var in self.successor.iter() {
            context.register(var.clone(), DomainEvents::ASSIGN);
        }
        Ok(())
    }
}
