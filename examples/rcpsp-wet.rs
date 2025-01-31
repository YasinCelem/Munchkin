//! Sets up munchkin with a model for the rcpsp weighted earliness/tardiness problem (rcpsp-wet).
//!
//! # Model
//! - Precedence constraints
//! - Cumulative constraints
//! - Minimise weighted earliness/tardiness; i.e. we have a desired start time for each task and we
//!   pay a cost for each time unit the task is early/late
use std::cmp::max;
use std::collections::HashSet;

use clap::ValueEnum;
use dzn_rs::DataFile;
use dzn_rs::ShapedArray;
use munchkin::branching::branchers::independent_variable_value_brancher::IndependentVariableValueBrancher;
use munchkin::branching::Brancher;
use munchkin::branching::InDomainMin;
use munchkin::branching::InputOrder;
use munchkin::model::Constraint;
use munchkin::model::IntVariable;
use munchkin::model::IntVariableArray;
use munchkin::model::Model;
use munchkin::model::Output;
use munchkin::model::VariableMap;
use munchkin::runner::Problem;
use munchkin::Solver;

munchkin::entry_point!(problem = RcpspWet, search_strategies = SearchStrategies);

#[derive(Clone, Copy, Default, ValueEnum)]
enum SearchStrategies {
    #[default]
    Default,
}

struct RcpspWet {
    start_times: IntVariableArray,
    weighted_earliness_tardiness: IntVariable,
}

impl Problem<SearchStrategies> for RcpspWet {
    fn create(data: DataFile<i32>) -> anyhow::Result<(Self, Model)> {
        let mut model = Model::default();

        let num_tasks = data
            .get::<i32>("n_tasks")
            .copied()
            .ok_or_else(|| anyhow::anyhow!("Missing int 'n_tasks' in data file."))?;
        let num_tasks_usize = usize::try_from(num_tasks)?;

        let num_resources = data
            .get::<i32>("n_res")
            .copied()
            .ok_or_else(|| anyhow::anyhow!("Missing int 'n_res' in data file."))?;
        let num_resources_usize = usize::try_from(num_resources)?;

        let durations = data
            .array_1d::<i32>("d", num_tasks_usize)
            .ok_or_else(|| anyhow::anyhow!("Missing int array 'd' in data file."))?;
        let durations: Vec<_> = iterate(durations)
            .copied()
            .map(u32::try_from)
            .collect::<Result<_, _>>()?;

        let resource_requirements = data
            .array_2d::<i32>("rr", [num_resources_usize, num_tasks_usize])
            .ok_or_else(|| anyhow::anyhow!("Missing 2d int array 'rr' in data file."))?;

        let resource_capacities = data
            .array_1d::<i32>("rc", num_resources_usize)
            .ok_or_else(|| anyhow::anyhow!("Missing int array 'rc' in data file."))?;

        let successors = data
            .array_1d::<HashSet<i32>>("suc", num_tasks_usize)
            .ok_or_else(|| anyhow::anyhow!("Missing set of int array 'suc' in data file."))?;

        let horizon = data
            .get::<i32>("t_max")
            .copied()
            .ok_or_else(|| anyhow::anyhow!("Missing int 't_max' in data file."))?;

        // The deadlines hold 3 values for each task:
        // - At position 0 - The desired start time of the task
        // - At position 1 - The cost per time unit that the task is early
        // - At position 2 - The cost per time unit that the task is late
        let deadline = data
            .array_2d::<i32>("deadline", [num_tasks as usize, 3])
            .ok_or_else(|| anyhow::anyhow!("Missing 2d int array 'deadline' in data file."))?;

        let start_times =
            model.new_interval_variable_array("Start", 0, horizon - 1, num_tasks_usize);

        for resource in 0..num_resources_usize {
            let resource_capacity = resource_capacities
                .get([resource])
                .copied()
                .unwrap()
                .try_into()?;

            let resource_requirements: Vec<_> = slice_row(resource_requirements, resource)
                .into_iter()
                .map(u32::try_from)
                .collect::<Result<_, _>>()?;

            let start_times = start_times.as_array(&model).collect();
            model.add_constraint(Constraint::Cumulative {
                start_times,
                durations: durations.clone(),
                resource_requirements,
                resource_capacity,
            });
        }

        let start_times_array: Vec<_> = start_times.as_array(&model).collect();
        for task in 0..num_tasks_usize {
            let task_successors = successors.get([task]).unwrap();

            for successor in task_successors.iter() {
                // The instance is 1-indexed.
                let successor = *successor as usize - 1;

                // Start[task] + Duration[task] <= Start[successor]
                model.add_constraint(Constraint::LinearLessEqual {
                    terms: vec![
                        start_times_array[task],
                        start_times_array[successor].scaled(-1),
                    ],
                    rhs: -(durations[task] as i32),
                });
            }
        }

        let maximum_objective_value = (0..num_tasks_usize)
            .map(|task| {
                max(
                    deadline.get([task, 1]).unwrap() * deadline.get([task, 0]).unwrap(),
                    deadline.get([task, 2]).unwrap() * (horizon - deadline.get([task, 0]).unwrap()),
                )
            })
            .sum();
        let objective = model.new_interval_variable("Objective", 0, maximum_objective_value);
        let zero_variable = model.new_interval_variable("Zero", 0, 0);
        let objective_variables = (0..num_tasks_usize).flat_map(|task| {
            // Note that we need to check whether we are scaling by 0 as otherwise the AffineView
            // will complain
            let earliness = (*deadline.get([task, 1]).unwrap() != 0).then(|| {
                let earliness = model.new_interval_variable(
                    format!("Earliness{task}"),
                    deadline.get([task, 0]).unwrap() - horizon,
                    *deadline.get([task, 0]).unwrap(),
                );
                // earliness = deadline[task, 0] - s[task]
                model.add_constraint(Constraint::LinearEqual {
                    terms: vec![earliness, start_times_array[task]],
                    rhs: *deadline.get([task, 0]).unwrap(),
                });
                let maximum_earliness_variable = model.new_interval_variable(
                    format!("MaximumEarliness{task}"),
                    0,
                    *deadline.get([task, 0]).unwrap(),
                );
                // maximum_earliness_variable = max(0, earliness)
                model.add_constraint(Constraint::Maximum {
                    terms: vec![zero_variable, earliness],
                    rhs: maximum_earliness_variable,
                });

                maximum_earliness_variable
            });

            let tardiness = (*deadline.get([task, 2]).unwrap() != 0).then(|| {
                let tardiness = model.new_interval_variable(
                    format!("tardiness{task}"),
                    -deadline.get([task, 0]).unwrap(),
                    horizon,
                );
                // tardiness = s[task] - deadline[i, 0]
                model.add_constraint(Constraint::LinearEqual {
                    terms: vec![tardiness, start_times_array[task].scaled(-1)],
                    rhs: -*deadline.get([task, 0]).unwrap(),
                });
                let maximum_tardiness_variable =
                    model.new_interval_variable(format!("MaximumTardiness{task}"), 0, horizon);
                // maximum_tardiness_variable = max(0, tardiness)
                model.add_constraint(Constraint::Maximum {
                    terms: vec![zero_variable, tardiness],
                    rhs: maximum_tardiness_variable,
                });
                maximum_tardiness_variable
            });

            // Scale the variables by the cost per time unit for earliness and tardiness
            [
                earliness.map(|earliness| earliness.scaled(*deadline.get([task, 1]).unwrap())),
                tardiness.map(|tardiness| tardiness.scaled(*deadline.get([task, 2]).unwrap())),
            ]
            .into_iter()
            .flatten()
        });

        // weighted_tardiness = ∑ deadline[task, 1] * max(0, deadline[task, 0] - s[task]) +
        // deadline[task, 2] * max(0, s[task] - deadline[task, 0])
        let objective_equality = Constraint::LinearEqual {
            terms: objective_variables
                .chain(std::iter::once(objective.scaled(-1)))
                .collect(),
            rhs: 0,
        };
        model.add_constraint(objective_equality);

        Ok((
            RcpspWet {
                start_times,
                weighted_earliness_tardiness: objective,
            },
            model,
        ))
    }

    fn objective(&self) -> IntVariable {
        self.weighted_earliness_tardiness
    }

    fn get_search(
        &self,
        strategy: SearchStrategies,
        _: &Solver,
        solver_variables: &VariableMap,
    ) -> impl Brancher + 'static {
        match strategy {
            SearchStrategies::Default => IndependentVariableValueBrancher::new(
                InputOrder::new(
                    solver_variables
                        .get_array(self.start_times)
                        .into_iter()
                        .chain([
                            solver_variables.to_solver_variable(self.weighted_earliness_tardiness)
                        ])
                        .collect(),
                ),
                InDomainMin,
            ),
        }
    }

    fn get_output_variables(&self) -> impl Iterator<Item = Output> + '_ {
        [
            Output::Array(self.start_times),
            Output::Variable(self.weighted_earliness_tardiness),
        ]
        .into_iter()
    }
}

fn iterate<T>(array: &ShapedArray<T, 1>) -> impl Iterator<Item = &T> {
    let [len] = *array.shape();

    (0..len).map(|i| array.get([i]).unwrap())
}

/// Extract a row from the 2d array.
fn slice_row(array: &ShapedArray<i32, 2>, row: usize) -> Vec<i32> {
    let [_, n_cols] = *array.shape();

    (0..n_cols)
        .map(move |col| {
            array
                .get([row, col])
                .copied()
                .expect("index is within range")
        })
        .collect()
}
