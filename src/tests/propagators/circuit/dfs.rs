#![cfg(test)]
use crate::engine::test_helper::TestSolver;
use crate::propagators::circuit::DfsCircuitPropagator;

#[test]
fn detects_failure() {
    let mut solver = TestSolver::default();

    let a = solver.new_variable(2, 2);
    let b = solver.new_variable(1, 1);
    let c = solver.new_variable(1, 2);

    let _ = solver
        .new_propagator(DfsCircuitPropagator::new([a, b, c].into()))
        .expect_err("Expected circuit to detect cycle");
}

#[test]
// An example based on Figure 4 in "Explaining circuit propagation - Francis & Stuckey (2013)"
fn detect_simple_dfs() {
    let mut solver = TestSolver::default();

    let a = solver.new_sparse_variable(&[2, 5, 6]);
    let b = solver.new_sparse_variable(&[3, 4]);
    let c = solver.new_sparse_variable(&[1]);
    let d = solver.new_sparse_variable(&[3]);
    let e = solver.new_sparse_variable(&[2, 4]);
    let f = solver.new_sparse_variable(&[1, 7]);
    let g = solver.new_sparse_variable(&[4, 5]);

    let _ = solver
        .new_propagator(DfsCircuitPropagator::new([a, b, c, d, e, f, g].into()))
        .expect("{Expected no error}");

    assert!(!solver.contains(f, 1));
    assert!(!solver.contains(g, 4));
}
