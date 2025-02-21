#![cfg(test)]
use crate::engine::test_helper::TestSolver;
use crate::propagators::cumulative::EnergeticReasoningPropagator;

// TODO: Add tests here.

/// A test case based on Figure 1 from "A quadratic edge-finding filtering algorithm for cumulative
/// resource constraints - Kameugne et al. (2014)"
#[test]
fn energetic_reasoning_updates_lower_bound() {
    let mut solver = TestSolver::default();

    let a = solver.new_variable(1, 4);
    let b = solver.new_variable(5, 5);
    let c = solver.new_variable(1, 7);
    let d = solver.new_variable(1, 5);
    let e = solver.new_variable(5, 6);
    let f = solver.new_variable(0, 9);

    let start_times = [a, b, c, d, e, f];
    let processing_times = [4, 2, 1, 3, 1, 7];
    let resource_usages = [2, 2, 2, 1, 1, 1];
    let capacity = 3;

    let _ = solver
        .new_propagator(EnergeticReasoningPropagator::new(
            start_times.into(),
            processing_times.into(),
            resource_usages.into(),
            capacity,
        ))
        .expect("Expected no conflict to occur");
    assert!(solver.lower_bound(f) >= 5);
}

/// A test case based on Figure 1 from "A quadratic edge-finding filtering algorithm for cumulative
/// resource constraints - Kameugne et al. (2014)"
#[test]
fn energetic_reasoning_updates_upper_bound() {
    let mut solver = TestSolver::default();

    let a = solver.new_variable(9, 12);
    let b = solver.new_variable(9, 11);
    let c = solver.new_variable(9, 15);
    let d = solver.new_variable(9, 13);
    let e = solver.new_variable(10, 11);
    let f = solver.new_variable(0, 10);

    let start_times = [a, b, c, d, e, f];
    let processing_times = [4, 2, 1, 3, 1, 7];
    let resource_usages = [2, 2, 2, 1, 1, 1];
    let capacity = 3;

    let _ = solver
        .new_propagator(EnergeticReasoningPropagator::new(
            start_times.into(),
            processing_times.into(),
            resource_usages.into(),
            capacity,
        ))
        .expect("Expected no conflict to occur");
    assert!(solver.upper_bound(f) <= 5);
}

/// A test case based on Figure 1 from "A quadratic edge-finding filtering algorithm for cumulative
/// resource constraints - Kameugne et al. (2014)"
#[test]
fn energetic_reasoning_finds_conflict() {
    let mut solver = TestSolver::default();

    let a = solver.new_variable(1, 4);
    let b = solver.new_variable(5, 5);
    let c = solver.new_variable(1, 7);
    let d = solver.new_variable(1, 5);
    let e = solver.new_variable(5, 6);
    let f = solver.new_variable(0, 4);

    let start_times = [a, b, c, d, e, f];
    let processing_times = [4, 2, 1, 3, 1, 7];
    let resource_usages = [2, 2, 2, 1, 1, 1];
    let capacity = 3;

    let _ = solver
        .new_propagator(EnergeticReasoningPropagator::new(
            start_times.into(),
            processing_times.into(),
            resource_usages.into(),
            capacity,
        ))
        .expect_err("Expected conflict to be detected");
}
