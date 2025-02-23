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
        // If a cycle is detected that doesn't cover all n nodes, the circuit is infeasible.
        for i in 0..n {
            if context.is_fixed(&self.successor[i]) {
                let mut current = i;
                let mut visited = vec![false; n];
                loop {
                    if visited[current] {
                        // A cycle is detected.
                        let cycle_size = visited.iter().filter(|&&v| v).count();
                        if cycle_size < n {
                            return Err(conjunction!().into());
                        }
                        break;
                    }
                    visited[current] = true;
                    // If the current successor is not fixed, we exit the DFS chain.
                    if !context.is_fixed(&self.successor[current]) {
                        break;
                    }
                    let next = context.lower_bound(&self.successor[current]);
                    if next == 0 {
                        // A candidate value of 0 is always invalid.
                        return Err(conjunction!().into());
                    }
                    // Convert from 1-indexed candidate to 0-indexed node.
                    current = (next - 1) as usize;
                }
            }
        }

        // --- Case 2: Unfixed Variables ---
        // For each unfixed variable, simulate the DFS chain for each candidate value.
        // For the starting variable, try each candidate (interpreted as 1-indexed),
        // then for subsequent nodes, use the lower_bound.
        // The simulation computes a “cycle size”: the number of distinct nodes reached
        // before either a cycle is encountered or an unfixed variable halts further simulation.
        // Then, we prune all candidate values except the one that yields the maximal cycle size,
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
                        candidate_results.push((candidate, 0));
                        continue;
                    }
                    let cycle_size = {
                        let mut visited = vec![false; n];
                        // Mark the starting node as visited.
                        visited[i] = true;
                        // Start simulation: candidate (1-indexed) becomes index candidate-1.
                        let mut current = (candidate - 1) as usize;
                        while !visited[current] {
                            visited[current] = true;
                            // For propagation, we use the lower_bound for subsequent nodes.
                            if !context.is_fixed(&self.successor[current]) {
                                // If we hit an unfixed variable, stop simulation.
                                break;
                            }
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
                    // Choose the candidate with the maximal cycle size.
                    // In case of ties, choose the candidate with the highest value.
                    let best_size = candidate_results.iter().map(|&(_, sz)| sz).max().unwrap();
                    let best_candidate = candidate_results
                        .iter()
                        .filter(|&&(_, sz)| sz == best_size)
                        .map(|&(cand, _)| cand)
                        .max()
                        .unwrap();
                    // Prune all other candidate values for this variable.
                    for (cand, _) in candidate_results {
                        if cand != best_candidate {
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
        // Register each successor variable for ASSIGN events.
        // This ensures that when a variable’s domain is reduced (or fixed),
        // propagation is re-triggered.
        for var in self.successor.iter() {
            init_context.register(var.clone(), DomainEvents::ASSIGN);
        }
        Ok(())
    }
}
