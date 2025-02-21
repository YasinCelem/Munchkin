#![cfg(test)]
use crate::engine::test_helper::TestSolver;
use crate::propagators::element::ElementPropagator;

#[test]
fn test_simple_propagation() {
    let mut solver = TestSolver::default();

    let x = solver.new_variable(0, 10);
    let y = solver.new_variable(0, 0);

    let index = solver.new_variable(1, 2);

    let rhs = solver.new_variable(5, 5);

    let propagator = solver
        .new_propagator(ElementPropagator::new(index, [x, y].into(), rhs))
        .expect("Expected no conflict here");

    // We know that the index can not point to y (since it is fixed at 0)
    // and the rhs is fixed at 5 so the index should be propagated to 0
    solver.assert_bounds(index, 1, 1);

    let result = solver.propagate(propagator);
    assert!(result.is_ok());

    // And the value of x should be fixed to 5 now
    solver.assert_bounds(x, 5, 5);
}

#[test]
fn test_simple_conflict() {
    let mut solver = TestSolver::default();

    let x = solver.new_variable(6, 10);
    let y = solver.new_variable(0, 4);

    let index = solver.new_variable(1, 2);

    let rhs = solver.new_variable(5, 5);

    // The instance we have provided is infeasible, the propagator should report a conflict
    let _ = solver
        .new_propagator(ElementPropagator::new(index, [x, y].into(), rhs))
        .expect_err("Expected conflict at the root level");
}
