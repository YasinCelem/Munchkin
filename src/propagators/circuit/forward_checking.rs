#![allow(unused, reason = "this file is a skeleton for the assignment")]

use crate::basic_types::PropagationStatusCP;
use crate::engine::cp::propagation::PropagationContextMut;
use crate::engine::cp::propagation::Propagator;
use crate::engine::cp::propagation::PropagatorInitialisationContext;
use crate::predicates::PropositionalConjunction;
use crate::variables::IntegerVariable;

//added 3 crates
use crate::conjunction;
use crate::engine::cp::domain_events::DomainEvents;
use crate::engine::cp::propagation::propagation_context::ReadDomains;

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
        //line above was first calles "DfsCircuit"
    }

    fn propagate(&self, mut context: PropagationContextMut) -> PropagationStatusCP {
        let n = self.successor.len();

        // case 1: fixedvariables
        // for all fixed variable, follow the chain using the fixed values
        // if an early cycle is detected return a conflict
        for i in 0..n {
            if context.is_fixed(&self.successor[i]) {
                let mut current = i;
                let mut visited = vec![false; n];
                while context.is_fixed(&self.successor[current]) {
                    if visited[current] {
                        // cycle detected --> count nodes in cycle.
                        let cycle_size = visited.iter().filter(|&&v| v).count();
                        if cycle_size < n {
                            // short cycle return propagation fails
                            return Err(conjunction!().into());
                        }
                        break;
                    }
                    //make it 1 index instead of 0 index
                    current = (context.lower_bound(&self.successor[current]) - 1) as usize;
                }
            }
        }

        // case 2: unfixed variables
        // for all unfixed variable, try candidate
        for i in 0..n {
            if !context.is_fixed(&self.successor[i]) {
                let lb = context.lower_bound(&self.successor[i]);
                let ub = context.upper_bound(&self.successor[i]);
                for candidate in lb..=ub {
                    // try a candidate for variable i.
                    let candidate_closes = {
                        // set node i as visited
                        let mut visited = vec![false; n];
                        visited[i] = true;
                        // index convert
                        let mut current = (candidate - 1) as usize;
                        // use lb strategy and use 0 index from 1 index
                        while current < n && !visited[current] {
                            visited[current] = true;
                            current = (context.lower_bound(&self.successor[current]) - 1) as usize;
                        }
                        // close candidate causing short cycle
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
