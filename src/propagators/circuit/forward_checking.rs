//AI usage: AI has been used to correct my code from 0 to 1 index and to find the right crates from the library. 


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
    
                while context.is_fixed(&self.successor[current]) {
                    let v = context.lower_bound(&self.successor[current]); 
                    let next = (v - 1) as usize; 
                    if visited.contains(&next) {
                        if chain.len() < n {
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
                if chain.len() < n {

                    let first = chain[0] as i32 + 1; 
                    let last = chain[chain.len() - 1];
                    context.remove(&self.successor[last], first, conjunction!())?;
                }

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
