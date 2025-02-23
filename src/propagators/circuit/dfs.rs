#![allow(unused, reason = "this file is a skeleton for the assignment")]

//! This module implements a DFS Circuit propagator.
//!
//! The propagator ensures that the circuit forms a valid DFS spanning all nodes.
//! It distinguishes between fixed and unfixed variables and uses DFS simulation
//! to prune inconsistent candidate values.

use crate::basic_types::PropagationStatusCP;
use crate::engine::cp::propagation::{
    PropagationContextMut, Propagator, PropagatorInitialisationContext,
};
use crate::predicates::PropositionalConjunction;
use crate::variables::IntegerVariable;
use crate::conjunction;
use crate::engine::cp::domain_events::DomainEvents;
use crate::engine::cp::propagation::propagation_context::ReadDomains;

/// Propagator enforcing a DFS circuit constraint.
///
/// The propagator checks that the circuit covers all nodes by considering two cases:
///
/// 1. **Fixed Variables:** For each fixed variable, it follows the chain defined by its
///    fixed (lower bound) value. If a cycle is detected that covers fewer than all nodes,
///    propagation fails.
///
/// 2. **Unfixed Variables:** For each unfixed variable, it simulates the DFS chain for every
///    candidate value (within the variable’s domain). It then retains only the candidate that
///    produces the maximal cycle size, breaking ties by choosing the highest candidate value.
pub(crate) struct DfsCircuitPropagator<Var> {
    /// Successor variables representing the circuit.
    successor: Box<[Var]>,
}

impl<Var> DfsCircuitPropagator<Var> {
    /// Creates a new DFS Circuit propagator.
    ///
    /// # Arguments
    ///
    /// * `successor` - A boxed slice of variables representing the successors in the circuit.
    pub(crate) fn new(successor: Box<[Var]>) -> Self {
        Self { successor }
    }
}

impl<Var: IntegerVariable + 'static> Propagator for DfsCircuitPropagator<Var> {
    fn name(&self) -> &str {
        "DfsCircuit"
    }

    /// Propagates the DFS circuit constraint.
    ///
    /// This method is divided into two main cases:
    ///
    /// **Case 1 (Fixed Variables):**
    /// - For each fixed variable, follow its chain using its fixed lower bound value.
    /// - If a cycle is detected that covers fewer than all nodes, a conflict is signaled.
    ///
    /// **Case 2 (Unfixed Variables):**
    /// - For each unfixed variable, simulate the DFS chain for every candidate value in its domain.
    /// - Each candidate is evaluated based on the number of distinct nodes visited before a cycle is reached.
    /// - All candidates except the one yielding the maximal cycle size (or the highest candidate in case of ties)
    ///   are pruned.
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
                        continue;
                    }
                    let cycle_size = {
                        let mut visited = vec![false; n];
                        visited[i] = true;
                        // Start simulation: candidate (1-indexed) becomes index candidate - 1.
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
                    // Choose the candidate with the maximal cycle size.
                    // In case of ties, choose the candidate with the highest value.
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

    /// Registers the successor variables for domain event notifications at the root.
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
