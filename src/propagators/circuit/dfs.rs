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
        // For each unfixed variable, we simulate the DFS chain for each candidate
        // (each candidate value actually in the variable’s domain, between lower_bound and upper_bound).
        // The simulation uses:
        //  - For the starting variable, the candidate value (interpreted as 1-indexed, so candidate v gives index v-1).
        //  - For subsequent variables, we always take the lower_bound (their fixed choice if fixed).
        // We record the cycle size (the number of distinct nodes visited before a cycle is encountered)
        // and then remove all candidates except the one with the maximal cycle size,
        // breaking ties by choosing the highest candidate value.
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
                    // Only prune if at least one candidate produces a full cycle.
                    let best_size = candidate_results.iter().map(|&(_, sz)| sz).max().unwrap();
                    if best_size == n {
                        // In case of ties, choose the candidate with the highest value.
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
                    // Otherwise, if no candidate yields a complete circuit, leave the domain untouched.
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
