#!/usr/bin/env python3
import subprocess

from common import Args, ModelType, check_runs, evaluate

def flatten(foo):
    result = []
    for x in foo:
        if hasattr(x, '__iter__') and not isinstance(x, str):
            for y in flatten(x):
               result.append(y) 
        else:
            result.append(x)
    return result

MODELS = ["tsp", "rcpsp"]

MODEL_TO_PROPAGATORS = {
        "tsp": [["forward-checking-circuit", "all-different", "element"]],
        "rcpsp": [["time-table-cumulative", "maximum"], ["energetic-reasoning-cumulative", "maximum"]]
}

# Timeout per instance.
INSTANCE_TIMEOUT = 20

PROPAGATOR_MODELS: dict[str, ModelType] = {
    "forward-checking-circuit": "tsp",
    "all-different": "tsp",
    "element": "tsp",
    "time-table-cumulative": "rcpsp",
    "energetic-reasoning-cumulative": "rcpsp",
    "maximum": "rcpsp",
}

PROPAGATOR_TEST_MODULES = {
    "forward-checking-circuit": "circuit::forward_checking",
    "all-different": "all_different",
    "element": "element",
    "time-table-cumulative": "cumulative::time_table",
    "energetic-reasoning-cumulative": "cumulative::energetic_reasoning",
    "maximum": "maximum",
}

PROPAGATOR_GRADE_CONTRIBUTION = {
    "forward-checking-circuit": 3,
    "all-different": 6,
    "element": 4,
    "time-table-cumulative": 3,
    "energetic-reasoning-cumulative": 6,
    "maximum": 3,
}

CONFLICT_ANALYSIS_TEST_MODULES = {
        "all-decision": "all_decision_learning",
        "unique-implication-point": "unique_implication_point"
}

CONFLICT_ANALYSIS_GRADE_CONTRIBUTION = {
        "all-decision": 5,
        "unique-implication-point": 5,
}

MINIMISATION_TEST_MODULES = {
        "recursive": "recursive_minimisation",
        "semantic": "semantic_minimisation"
}

MINIMISATION_GRADE_CONTRIBUTION = {
        "recursive": 5,
        "semantic": 5,
}


def grade_propagator(propagator: str) -> int:
    """Grade a single global propagator. Return the contribution to the final grade for this propagator."""

    print(f"Grading {propagator}...")

    # For each propagator, run `cargo test`
    test_filter = f"tests::propagators::{PROPAGATOR_TEST_MODULES[propagator]}"
    result = subprocess.run(
        ["cargo", "test", test_filter],
        capture_output=True,
        text=True,
    )

    if result.returncode != 0:
        print(result.stdout)
        return 0

    # For each propagator, run the associated model once with the propagator enabled. All solutions should be correct.
    context = evaluate(Args(
        model=PROPAGATOR_MODELS[propagator],
        timeout=INSTANCE_TIMEOUT,
        flags=["-G", propagator],
        allow_dirty=True,
        explanation_checks=True,
    ))
    if context is None:
        return 0

    if not check_runs(context):
        return 0

    print(f"  Passes all tests!")
    return PROPAGATOR_GRADE_CONTRIBUTION[propagator]

def grade_conflict_analysis(learning: str, propagators: str, model: str) -> int:
    """Grade a conflict analysis procedure given the propagators and model. Return the contribution to the final grade for this conflict analysis procedure."""

    test_filter = f"tests::conflict_analysis::{CONFLICT_ANALYSIS_TEST_MODULES[learning]}"
    result = subprocess.run(
        ["cargo", "test", test_filter],
        capture_output=True,
        text=True,
    )

    if result.returncode != 0:
        print(result.stdout)
        return 0

    context = evaluate(Args(
        model=model,
        timeout=INSTANCE_TIMEOUT,
        flags=flatten([["-G", propagator] for propagator in propagators] + ["-C", learning]),
        allow_dirty=True,
        explanation_checks=True,
    ))
    if context is None:
        return 0

    if not check_runs(context):
        return 0

    print(f"  Passes all tests!")
    return CONFLICT_ANALYSIS_GRADE_CONTRIBUTION[learning] / len(MODEL_TO_PROPAGATORS[model]) / len(MODELS)

def grade_minimisation(minimisation: str, propagators: str, model: str) -> int:
    """Grade a minimisation approach given the propagators and model. Return the contribution to the final grade for this nogood minimisation procedure."""

    test_filter = f"tests::minimisation::{MINIMISATION_TEST_MODULES[minimisation]}"
    result = subprocess.run(
        ["cargo", "test", test_filter],
        capture_output=True,
        text=True,
    )

    if result.returncode != 0:
        print(result.stdout)
        return 0

    context = evaluate(Args(
        model=model,
        timeout=INSTANCE_TIMEOUT,
        flags=flatten([["-G", propagator] for propagator in propagators] + ["-C", "unique-implication-point", "-M", minimisation]),
        allow_dirty=True,
        explanation_checks=True,
    ))
    if context is None:
        return 0

    if not check_runs(context):
        return 0

    print(f"  Passes all tests!")
    return MINIMISATION_GRADE_CONTRIBUTION[minimisation] / len(MODEL_TO_PROPAGATORS[model]) / len(MODELS)

def run():
    assert PROPAGATOR_MODELS.keys() == PROPAGATOR_TEST_MODULES.keys() and PROPAGATOR_MODELS.keys() == PROPAGATOR_GRADE_CONTRIBUTION.keys(), \
        "The keys for these dictionaries must be all the global names in the model executables."

    max_grade = sum(PROPAGATOR_GRADE_CONTRIBUTION.values()) + sum(CONFLICT_ANALYSIS_GRADE_CONTRIBUTION.values()) + sum(MINIMISATION_GRADE_CONTRIBUTION.values())
    assert max_grade == 45, \
        f"Expected the maximum total grade to be 45 points. Was {max_grade}"

    propagators = PROPAGATOR_MODELS.keys()

    total_grade = 0

    for propagator in propagators:
        total_grade += grade_propagator(propagator)

    for learning in CONFLICT_ANALYSIS_TEST_MODULES.keys(): 
        print(f"Grading {learning}...")
        for model in MODELS:
            for propagators in MODEL_TO_PROPAGATORS[model]:
                print(f"Evaluating {learning} with {model} - {propagators}")
                total_grade += grade_conflict_analysis(learning, propagators, model)

    for minimisation in MINIMISATION_TEST_MODULES.keys(): 
        print(f"Grading {minimisation}...")
        for model in MODELS:
            for propagators in MODEL_TO_PROPAGATORS[model]:
                print(f"Evaluating {minimisation} with {model} - {propagators}")
                total_grade += grade_minimisation(minimisation, ", ".join(propagators), model)

    print(f"Grade = {total_grade}% / {max_grade}%")


if __name__ == "__main__":
    run()
