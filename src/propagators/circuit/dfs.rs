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

pub(crate) struct DfsCircuitPropagator<Var> {
    successor: Box<[Var]>,
}

impl<Var> DfsCircuitPropagator<Var> {
    pub(crate) fn new(successor: Box<[Var]>) -> Self {
        Self { successor }
    }
}

impl<Var: IntegerVariable + 'static> Propagator for DfsCircuitPropagator<Var> {
    fn name(&self) -> &str {
        "DfsCircuit"
    }

    fn propagate(&self, mut context: PropagationContextMut) -> PropagationStatusCP {
        let n = self.successor.len();

        // case 1: fixed variabes
        // for all fixed variable, follow the chain using the fixed values
        // if an early cycle is detected return a conflict

        for i in 0..n {
            if context.is_fixed(&self.successor[i]) {
                let mut current = i;
                let mut visited = vec![false; n];
                while context.is_fixed(&self.successor[current]) {
                    if visited[current] {
                        let cycle_size = visited.iter().filter(|&&v| v).count();
                        if cycle_size < n {
                            return Err(conjunction!().into());
                        }
                        break;
                    }
                    visited[current] = true;
                    current = (context.lower_bound(&self.successor[current]) - 1) as usize;
                }
            }
        }

        // case 2: unfixed variables
        // for all unfixed variable, do a dfs extension for each 'candidate' value and return cyclesize
        // choose lower bound strategy until a cycle is encountered
        // remove lesser candidate for better performing candidate 
        // if more candidates have maximum cyclesize, use the candidate with the highest value.
        for i in 0..n {
            if !context.is_fixed(&self.successor[i]) {
                let lb = context.lower_bound(&self.successor[i]);
                let ub = context.upper_bound(&self.successor[i]);
                let mut candidate_results = Vec::new();
                // iterate over the entire variable interval domain
                for candidate in lb..=ub {
                    if !context.contains(&self.successor[i], candidate) {
                        continue;
                    }
                    // use dfs starting with candidate at variable i.
                    let cycle_size = {
                        let mut visited = vec![false; n];
                        visited[i] = true;
                        let mut current = (candidate - 1) as usize;
                        // follow the lower bound even if the variable is unfixed.
                        while !visited[current] {
                            visited[current] = true;
                            current = (context.lower_bound(&self.successor[current]) - 1) as usize;
                        }
                        visited.iter().filter(|&&v| v).count()
                    };
                    candidate_results.push((candidate, cycle_size));
                }
                // determine and choose best cyclesize
                if !candidate_results.is_empty() {
                    let best_size = candidate_results.iter().map(|&(_, sz)| sz).max().unwrap();
                    let best_candidate = candidate_results
                        .iter()
                        .filter(|&&(_, sz)| sz == best_size)
                        .map(|&(cand, _)| cand)
                        .max()
                        .unwrap();
                    // remove lesser candidates for better performing candidate
                    for (cand, sz) in candidate_results {
                        if sz < best_size || (sz == best_size && cand < best_candidate) {
                            context.remove(&self.successor[i], cand, conjunction!())?;
                        }
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
