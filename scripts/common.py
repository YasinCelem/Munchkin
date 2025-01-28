from dataclasses import dataclass
from pathlib import Path
from typing import Iterable, Literal, Union
from subprocess import run
from enum import Enum
import shutil
import json


MODELS = ["tsp", "rcpsp-makespan", "rcpsp-tardiness"]

ModelType = Union[Literal["tsp"], Literal["rcpsp-makespan"], Literal["rcpsp-tardiness"]] 

DATA_DIR = (Path(__file__).parent / ".." / "data").resolve()
EXPERIMENT_DIR = (Path(__file__).parent / ".." / "experiments").resolve()

INSTANCES = {
    "tsp": (DATA_DIR / "tsp"),
    "rcpsp-makespan": (DATA_DIR / "rcpsp"),
    "rcpsp-tardiness": (DATA_DIR / "rcpsp"),
}

MINIZINC_MODELS = {
    "tsp": (Path(__file__).parent / ".." / "models" / "tsp.mzn").resolve(),
    "rcpsp-makespan": (Path(__file__).parent / ".." / "models" / "rcpsp-makespan.mzn").resolve(),
    "rcpsp-tardiness": (Path(__file__).parent / ".." / "models" / "tsp.mzn").resolve(),
}

SOLUTION_SEPARATOR = "-" * 10
OPTIMALITY_PROVEN = "=" * 10

class RunError(Enum):
    IncorrectSolution = 1,
    IncorrectOptimalSolution = 2,
    Both = 3,

class bcolors:
    OKGREEN = '\033[92m'
    FAIL = '\033[91m'
    ENDC = '\033[0m'


@dataclass
class Context:
    """The context for the current evaluation."""

    directory: Path
    """The path of the current evaluation runs."""

    runs: Path
    """The directory containing the log files."""

    model: ModelType
    """The model being evaluated."""

    timeout: int
    """The timeout to give to each instance."""
    

    def to_dict(self) -> dict:
        """Turn the context into a JSON-serializable dictionary."""

        return {
            "directory": str(self.directory),
            "runs": str(self.runs),
            "model": self.model,
            "timeout": self.timeout,
        }


    @classmethod
    def from_dict(cls, dictionary: dict) -> 'Context':
        """Create a Context from a JSON-dictionary."""

        return cls(
            directory=Path(dictionary["directory"]),
            runs=Path(dictionary["runs"]),
            model=dictionary["model"],
            timeout=int(dictionary["timeout"]),
        )



def check_runs(context: Context):
    wrong_solution_instances = []
    wrong_optimality_instances = []
    num_instances = 0

    with open(INSTANCES[context.model] / "optimal_values.json", "r") as file:
        optimal_values = json.load(file)

    for run in context.runs.iterdir():
        num_instances += 1
        run_status = check_run(run, context.model, optimal_values)
        if run_status is RunError.IncorrectSolution:
            wrong_solution_instances.append(run.stem)
        elif run_status is RunError.IncorrectOptimalSolution:
            wrong_optimality_instances.append(run.stem)
        elif run_status is RunError.Both:
            wrong_solution_instances.append(run.stem)
            wrong_optimality_instances.append(run.stem)

    if len(wrong_solution_instances) > 0:
        print(f"\n{bcolors.FAIL}{len(wrong_solution_instances)}/{num_instances} instances reported at least one incorrect solution{bcolors.ENDC}")
        for errored_instance in wrong_solution_instances:
            print(f"{bcolors.FAIL}\t{errored_instance}{bcolors.ENDC}")
    if len(wrong_optimality_instances) > 0:
        print(f"\n{bcolors.FAIL}{len(wrong_optimality_instances)}/{num_instances} instances reported incorrect optimality{bcolors.ENDC}")
        for wrong_optimality_instance in wrong_optimality_instances:
            print(f"{bcolors.FAIL}\t{wrong_optimality_instance}{bcolors.ENDC}")


def check_run(run: Path, model: ModelType, optimal_values: dict) -> bool:
    """
        Returns true if there were errors
    """
    instance_name = run.stem

    print(f"Checking {instance_name} for {model}")

    dzn_dir = generate_dzn_instances(run)

    model_path = MINIZINC_MODELS[model]
    data_path = INSTANCES[model] / f"{instance_name}.dzn"

    wrong_optimality = False
    wrong_solution = False

    if is_optimal(run): 
        reported_optimal_value = next((int(line.removeprefix("%%  objective=")) for line in list(iter_solutions(run))[-1].splitlines() if line.startswith("%%  objective=")), None)
        if optimal_values[instance_name] != reported_optimal_value:
            wrong_optimality = True
            print(f"{bcolors.FAIL}Incorrect optimality recorded for {instance_name}; expected {optimal_values[instance_name]} but was {reported_optimal_value}\n{bcolors.ENDC}")

    wrong_solution = run_minizinc(model_path, data_path, dzn_dir)

    if wrong_solution and wrong_optimality:
        return RunError.Both
    elif wrong_solution:
        return RunError.IncorrectSolution
    elif wrong_optimality:
        return RunError.IncorrectOptimalSolution
    else:
        None


def run_minizinc(model_path: Path, data_path: Path, solutions: Path) -> bool:
    """
        Returns true if there were errors
    """
    error_count = 0
    solution_count = 0

    for instance in solutions.iterdir():
        solution_count += 1
        mzn_command_args = ["minizinc", "--solver", "cp-sat", str(model_path), str(data_path), str(instance)]
        result = run(mzn_command_args, capture_output=True, text=True)

        if result.returncode != 0:
            print("STDOUT:")
            print(result.stdout)

            print("\nSTDERR:")
            print(result.stderr)

            cmd = " ".join(mzn_command_args)
            raise Exception(f"MiniZinc failed. Command: {cmd}")

        if "UNSATISFIABLE" in result.stdout:
            error_count += 1
            print(f"{bcolors.FAIL}Error detected in solution{bcolors.ENDC}")
            print(f"  Solution: {instance.stem}")
            print(f"  Model: {model_path.stem}")
            print(f"  Data: {data_path.stem}")

    if error_count > 0:
        print(f"{bcolors.FAIL}\n{error_count}/{solution_count} solutions had errors.{bcolors.ENDC}")
        return True
    elif solution_count > 0:
        print(f"{bcolors.OKGREEN}All solutions are correct!{bcolors.ENDC}")
        return False
    else:
        print(f"No reported solutions!")
        return False


def iter_solutions(run: Path) -> Iterable[str]:
    """Iterate over the individual solutions of a run."""

    output_log_path = run / "output.log"

    with output_log_path.open('r') as output:
        output = output.read()

    if "UNSATISFIABLE" in output or "UNKNOWN" in output:
        # There are no solutions in this file.
        return iter([])

    return filter(
        lambda s: s != "" and s != OPTIMALITY_PROVEN, 
        map(lambda s: s.strip(), output.split(SOLUTION_SEPARATOR))
    )

def is_optimal(run: Path) -> bool:
    output_log_path = run / "output.log"

    with output_log_path.open('r') as output:
        output = output.read()

    return OPTIMALITY_PROVEN in output


def generate_dzn_instances(run: Path) -> Path:
    """
    For all the reported solutions in this run, generate a DZN file.
    """

    solutions_dir = run / "solutions_dzn"
    if solutions_dir.is_dir():
        # We delete previous solutions if they are generated.
        shutil.rmtree(solutions_dir)

    solutions_dir.mkdir()

    solutions = list(iter_solutions(run))

    print(f"  Identified {len(solutions)} solution(s).")

    for idx, solution in enumerate(solutions):
        solution = solution.strip()

        if solution == "":
            continue

        solution_file = solutions_dir / f"sol-{idx}.dzn"

        with solution_file.open('w') as solution_file:
            for line in solution.split("\n"):
                # Disregard lines (aka variables) that start with an '_' as they are not 
                # variables that correspond to the MZN model.
                if line.startswith("_"):
                    continue

                solution_file.write(f"{line}\n")

    return solutions_dir
