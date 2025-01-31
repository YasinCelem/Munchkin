#!/usr/bin/env python3

from argparse import ArgumentParser
from dataclasses import dataclass
from pathlib import Path
from subprocess import run
from datetime import datetime
import json
import sys

from common import *


def todo(message: str):
    print(f"TODO: {message}")
    sys.exit(1)



@dataclass
class Args:
    """The arguments passed to the evaluation script."""

    model: ModelType
    """The model to evaluate."""

    timeout: int
    """The timeout to give to each instance."""

    flags: List[str]
    """Additional flags provided to the solver."""

    allow_dirty: bool
    """If true, allows uncommitted git changes."""


@dataclass 
class GitStatus:
    has_dirty_files: bool
    commit_hash: str


def get_git_status(args: Args) -> GitStatus:
    """
    Get the current git hash of the project. If there are uncommitted changes,
    the program will exit.
    """

    status_result = run(
        ["git", "status", "--porcelain"], 
        capture_output=True, 
        text=True
    )
    if status_result.returncode != 0:
        print(f"Failed to run `git status`")
        print(status_result.stderr)
        sys.exit(1)

    commit_hash_result = run(
        ["git", "rev-parse", "HEAD"], 
        capture_output=True, 
        text=True
    )
    if commit_hash_result.returncode != 0:
        print(f"Failed to run `git rev-parse HEAD`")
        print(commit_hash_result.stderr)
        sys.exit(1)

    commit_hash = commit_hash_result.stdout.strip()

    if len(status_result.stdout) == 0:
        return GitStatus(
            commit_hash=commit_hash, 
            has_dirty_files=False,
        )

    if args.allow_dirty:
        return GitStatus(
            commit_hash=commit_hash, 
            has_dirty_files=True,
        )

    print(f"There are uncommitted changes in the project. Cancelling evaluation.")
    print(f"To ignore this error, run with the '--allow-dirty' flag.")
    sys.exit(1)


def compile_executable(args: Args, experiment_dir: Path) -> Path:
    """
    Compiles the executable and returns a Path to it.
    """

    result = run(["cargo", "build", "--release", "--example", args.model])

    if result.returncode != 0:
        # The cargo output is forwarded so no need for an extra message here.
        sys.exit(1)

    on_windows = sys.platform.startswith("win")
    executable_name = f"{args.model}.exe" if on_windows else args.model

    executable_path = Path("target/release/examples/") / executable_name
    shutil.copy2(executable_path, experiment_dir)

    return experiment_dir / executable_name

def initialise(args: Args) -> Context:
    """
    Prepare everything for evaluation.
    """

    # Create the directory which stores all the logs, if it does not exist.
    EXPERIMENT_DIR.mkdir(exist_ok=True)

    git_status = get_git_status(args)

    # Create a directory for the current evaluation.
    timestamp = datetime.now().strftime("%Y%m%d-%H.%M.%S.%f")
    directory = EXPERIMENT_DIR / f"{timestamp}-{args.model}"

    try:
        directory.mkdir()
    except FileExistsError:
        print("Error creating evaluation context directory. Please try again.")
        sys.exit(1)

    # Create the directory which will contain all the solver logs.
    runs = directory / "runs"
    runs.mkdir()

    executable = compile_executable(args, directory)

    context = Context(
        directory=directory, 
        runs=runs,
        model=args.model,
        timeout=args.timeout,
        commit_hash=git_status.commit_hash,
        executable=executable,
        flags=args.flags,
        has_dirty_files=git_status.has_dirty_files,
    )

    with (directory / "manifest.json").open('w') as context_file:
        json.dump(context.to_dict(), context_file, indent=4)

    return context


def run_instances(context: Context):
    instances = INSTANCES[context.model]

    for instance in instances.glob("*.dzn"):
        run_instance(instance, context)


def run_instance(instance: Path, context: Context):
    print(f"Evaluating instance {instance.stem}")

    instance_directory = context.runs / instance.stem
    instance_directory.mkdir()

    log_file_path = instance_directory / "output.log"
    err_file_path = instance_directory / "output.err"

    with log_file_path.open('w') as log_file:
        with err_file_path.open('w') as err_file:
            run(
                [context.executable, instance, "solve", *context.flags, str(context.timeout)],
                stdout=log_file,
                stderr=err_file,
            ) 


def evaluate(args: Args):
    context = initialise(args)

    run_instances(context)


if __name__ == "__main__":
    arg_parser = ArgumentParser(description="Evaluate a Munckin model")
     
    arg_parser.add_argument("model", help="The model to evaluate.", choices=MODELS)
    arg_parser.add_argument("timeout", help="Time budget for every instance in seconds.", type=int)
    arg_parser.add_argument(
         "--allow-dirty", 
         action="store_true", 
         help="Allow uncommitted files when running the experiment.",
    )
    arg_parser.add_argument("model_flags", nargs='*', help="Arguments after --")

    args = arg_parser.parse_args()

    evaluate(Args(
        model=args.model,
        timeout=args.timeout,
        flags=args.model_flags,
        allow_dirty=args.allow_dirty,
    ))
