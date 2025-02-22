#![allow(unused, reason = "this file is a skeleton for the assignment")]

/*!
# Generative AI Usage

I used generative AI tools (e.g., ChatGPT) during the development of this file for:
1. **Outline of algorithms**:
   - To generate an initial high‑level pseudocode for the DFS circuit propagator.
2. **Code refinement**:
   - To suggest improvements in function structure, variable naming, and documentation once the core logic was implemented.
*/

//! This module implements a DFS circuit propagator that enforces circuit constraints
//! by exploring fixed and unfixed variables and pruning inconsistent candidate assignments.

use crate::basic_types::PropagationStatusCP;
use crate::conjunction;
use crate::engine::cp::domain_events::DomainEvents;
use crate::engine::cp::propagation::propagation_context::ReadDomains;
use crate::engine::cp::propagation::{
    PropagationContextMut, Propagator, PropagatorInitialisationContext,
};
use crate::predicates::PropositionalConjunction;
use crate::variables::IntegerVariable;

/// Represents a DFS circuit propagator enforcing circuit constraints.
pub(crate) struct DfsCircuitPropagator<Var> {
    successor: Box<[Var]>,
}

impl<Var> DfsCircuitPropagator<Var> {
    /// Constructs a new DFS circuit propagator with the provided successor array.
    pub(crate) fn new(successor: Box<[Var]>) -> Self {
        Self { successor }
    }
}

impl<Var: IntegerVariable + 'static> Propagator for DfsCircuitPropagator<Var> {
    /// Returns the name of the propagator.
    fn name(&self) -> &str {
        "DfsCircuit"
    }

    /// Propagates the circuit constraint by processing fixed and unfixed variables.
    ///
    /// # Propagation Details
    ///
    /// - **Case 1: Fixed Variables**
    ///   - For each fixed variable, the propagator follows its chain using its lower bound value.
    ///   - If a cycle is detected that does not cover all nodes, propagation fails.
    ///
    /// - **Case 2: Unfixed Variables**
    ///   - For each unfixed variable, the DFS chain is simulated for each candidate in its domain.
    ///   - The candidate with the maximal cycle size (or highest value in case of ties) is retained,
    ///     and other candidates are pruned.
    fn propagate(&self, mut context: PropagationContextMut) -> PropagationStatusCP {
        let n = self.successor.len();

        // --- Case 1: Fixed Variables ---
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
                    // Convert candidate value (1-indexed) to 0-index.
                    current = (next - 1) as usize;
                }
            }
        }

        // --- Case 2: Unfixed Variables ---
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
            }
        }

        Ok(())
    }

    /// Initialises the propagator at the root of the search tree.
    ///
    /// Registers each successor variable with the event type `ASSIGN`.
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
