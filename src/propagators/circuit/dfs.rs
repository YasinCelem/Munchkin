#![allow(unused, reason = "this file is a skeleton for the assignment")]

use crate::basic_types::PropagationStatusCP;
use crate::engine::cp::propagation::{
    PropagationContextMut, Propagator, PropagatorInitialisationContext,
};
use crate::predicates::PropositionalConjunction;
use crate::variables::IntegerVariable;
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

        // --- Case 1: Fixed Variables ---
        // For each fixed variable, follow its chain using its fixed (lower bound) value.
        // The candidate values are 1-indexed; we subtract 1 to obtain an index.
        // If the cycle found covers fewer than n nodes, propagation fails.
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
                    let next = context.lower_bound(&self.successor[current]);
                    if next == 0 {
                        // A candidate value of 0 is invalid.
                        return Err(conjunction!().into());
                    }
                    // Convert candidate value to 0-index.
                    current = (next - 1) as usize;
                }
            }
        }

        // --- Case 2: Unfixed Variables ---
        // For each unfixed variable, simulate the DFS chain for each candidate value in its domain.
        // Record the cycle size (i.e. the number of distinct nodes visited before a cycle is encountered)
        // and then prune (remove) all candidates except the one with the maximal cycle size.
        // However, we only prune if at least one candidate produces a complete circuit (cycle size == n).
        for i in 0..n {
            if !context.is_fixed(&self.successor[i]) {
                let lb = context.lower_bound(&self.successor[i]);
                let ub = context.upper_bound(&self.successor[i]);
                let mut candidate_results = Vec::new();
                for candidate in lb..=ub {
                    if !context.contains(&self.successor[i], candidate) {
                        continue;
                    }
                    // Guard: candidate value 0 is always invalid.
                    if candidate == 0 {
                        continue;
                    }
                    let cycle_size = {
                        let mut visited = vec![false; n];
                        visited[i] = true;
                        // Start simulation: candidate (1-indexed) becomes index candidate-1.
                        let mut current = (candidate - 1) as usize;
                        while !visited[current] {
                            visited[current] = true;
                            let next = context.lower_bound(&self.successor[current]);
                            if next == 0 {
                                break;
                            }
                            current = (next - 1) as usize;
                        }
                        visited.iter().filter(|&&v| v).count()
                    };
                    candidate_results.push((candidate, cycle_size));
                }
                if !candidate_results.is_empty() {
                    let best_size = candidate_results.iter().map(|&(_, sz)| sz).max().unwrap();
                    // Only prune if a candidate produces a complete circuit.
                    if best_size == n {
                        let best_candidate = candidate_results
                            .iter()
                            .filter(|&&(_, sz)| sz == best_size)
                            .map(|&(cand, _)| cand)
                            .max()
                            .unwrap();
                        for (cand, _) in candidate_results {
                            if cand != best_candidate {
                                context.remove(&self.successor[i], cand, conjunction!())?;
                            }
                        }
                    }
                    // If no candidate produces a complete circuit, we leave the domain unchanged.
                }
            }
        }

        Ok(())
    }

    fn initialise_at_root(
        &mut self,
        init_context: &mut PropagatorInitialisationContext,
    ) -> Result<(), PropositionalConjunction> {
        for var in self.successor.iter() {
            init_context.register(var.clone(), DomainEvents::ASSIGN);
        }
        Ok(())
    }
}
