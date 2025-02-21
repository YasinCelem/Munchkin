#![cfg(test)]

use crate::engine::test_helper::TestSolver;
use crate::propagators::all_different::AllDifferentPropagator;

#[test]
fn test_bounds_propagation() {
    let mut solver = TestSolver::default();

    let x1 = solver.new_variable(0, 3);
    let x2 = solver.new_variable(0, 3);
    let x3 = solver.new_variable(0, 3);
    let x4 = solver.new_variable(1, 2);
    let x5 = solver.new_variable(-2, 6);
    let x6 = solver.new_variable(1, 6);

    let variables = [x1, x2, x3, x4, x5, x6];

    let _ = solver
        .new_propagator(AllDifferentPropagator::new(variables.into()))
        .expect("Expected no error");

    solver.assert_domain(x5, vec![-2, -1, 4, 5, 6]);
    solver.assert_bounds(x6, 4, 6);
}

#[test]
fn test_holes_are_punched() {
    // Taken from https://www.lnmb.nl/conferences/2014/programlnmbconference/Shaw-1.pdf
    let mut solver = TestSolver::default();

    let x1 = solver.new_sparse_variable(&[1, 2]);
    let x2 = solver.new_sparse_variable(&[2, 3]);
    let x3 = solver.new_sparse_variable(&[1, 3]);
    let x4 = solver.new_sparse_variable(&[3, 4]);
    let x5 = solver.new_sparse_variable(&[2, 4, 5, 6]);
    let x6 = solver.new_sparse_variable(&[5, 6, 7]);

    let _ = solver
        .new_propagator(AllDifferentPropagator::new([x1, x2, x3, x4, x5, x6].into()))
        .expect("Expected no error");

    solver.assert_bounds(x4, 4, 4);
}
