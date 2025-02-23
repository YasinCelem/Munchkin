#![allow(unused, reason = "this file is a skeleton for the assignment")]

use core::task;
use std::cmp;
use std::collections::HashSet;

use crate::basic_types::PropagationStatusCP;
use crate::conjunction;
use crate::engine::cp::domain_events::DomainEvents;
use crate::engine::cp::propagation::PropagationContextMut;
use crate::engine::cp::propagation::Propagator;
use crate::engine::cp::propagation::PropagatorInitialisationContext;
use crate::engine::cp::propagation::propagation_context::ReadDomains;
use crate::predicates::PropositionalConjunction;
use crate::variables::IntegerVariable;

pub(crate) struct EnergeticReasoningPropagator<Var> {
    start_times: Box<[Var]>,
    durations: Box<[u32]>,
    resource_requirements: Box<[u32]>,
    resource_capacity: u32,
    // TODO: you can add more fields here!
}

impl<Var> EnergeticReasoningPropagator<Var> {
    pub(crate) fn new(
        start_times: Box<[Var]>,
        durations: Box<[u32]>,
        resource_requirements: Box<[u32]>,
        resource_capacity: u32,
    ) -> Self {
        EnergeticReasoningPropagator {
            start_times,
            durations,
            resource_requirements,
            resource_capacity,
        }
    }
}

impl<Var: IntegerVariable + 'static> Propagator for EnergeticReasoningPropagator<Var> {
    fn name(&self) -> &str {
        "EnergeticReasoning"
    }

    fn propagate(&self, mut context: PropagationContextMut) -> PropagationStatusCP {
        let mut interval_start_times = HashSet::new();
        let mut interval_end_times = HashSet::new();

        // Add interesting start and end times to be checked
        for task_i in 0..self.start_times.len() {
            let start_time = &self.start_times[task_i];
            let duration = self.durations[task_i];
            let resource_requirement = self.resource_requirements[task_i];

            // The interval within which the task is scheduled including its duration
            let interval_lb = context.lower_bound(start_time);
            let interval_ub = context.upper_bound(start_time);

            let _ = interval_start_times.insert(interval_lb);
            let _ = interval_end_times.insert(interval_ub + duration as i32 - 1);

            if interval_lb + duration as i32 - 1 >= interval_ub {
                let _ = interval_end_times.insert(interval_lb + duration as i32 - 1);
                let _ = interval_start_times.insert(interval_ub);
            }
        }

        for start_time in interval_start_times.iter() {
            for end_time in interval_end_times.iter() {
                if end_time < start_time { continue; }
                let mut energy_required = 0;

                for task_i in 0..self.start_times.len() {
                    let start_time_var = &self.start_times[task_i];
                    let duration = self.durations[task_i] as i32;
                    let resource_requirement = self.resource_requirements[task_i] as i32;

                    let task_start_time_lb = context.lower_bound(start_time_var);
                    let task_start_time_ub = context.upper_bound(start_time_var);

                    let forced_lb = task_start_time_ub;
                    let forced_ub = task_start_time_lb + duration - 1;

                    // The minimum required overlap is the min of the overlap if the task is scheduled as early as 
                    // possible, and the overlap if the task is scheduled as late as possible
                    let early_overlap = ((*end_time).min(task_start_time_lb + duration - 1) - (*start_time).max(task_start_time_lb) + 1).max(0);
                    let late_overlap = ((*end_time).min(task_start_time_ub + duration - 1) - (*start_time).max(task_start_time_ub) + 1).max(0);
                    let forced_overlap = early_overlap.min(late_overlap);

                    let forced_energy = forced_overlap * resource_requirement;
                    energy_required += forced_energy;
                }  

                // If there is enough not energy for all tasks within the time interval we have a conflict
                let energy_available = (end_time - start_time + 1) * self.resource_capacity as i32;
                if energy_required > energy_available { 
                    // I could not figure out how to properly return a conflict... 
                    // return Err(EmptyDomain); does not work for some reason
                    // So I do this for now
                    context.set_lower_bound(&self.start_times[0], 0, conjunction!())?;
                    context.set_upper_bound(&self.start_times[0], -1, conjunction!())?;
                }  

                // If there is enough energy we can check if there are other tasks which are forced to be outside the interval
                for task_i in 0..self.start_times.len() {
                    let start_time_var = &self.start_times[task_i];
                    let duration = self.durations[task_i] as i32;
                    let resource_requirement = self.resource_requirements[task_i] as i32;
                    let task_energy = duration * resource_requirement;

                    let task_start_time_lb = context.lower_bound(start_time_var);
                    let task_start_time_ub = context.upper_bound(start_time_var);

                    let early_overlap = ((*end_time).min(task_start_time_lb + duration - 1) - (*start_time).max(task_start_time_lb) + 1).max(0);
                    let late_overlap = ((*end_time).min(task_start_time_ub + duration - 1) - (*start_time).max(task_start_time_ub) + 1).max(0);
                    let forced_overlap = early_overlap.min(late_overlap);

                    let forced_energy = forced_overlap * resource_requirement;

                    // Remove the energy that was already added for this task
                    let energy_required_other_tasks = energy_required - forced_energy;

                    // Calculate maximum allowed overlap of task with interval
                    let maximum_overlap = if resource_requirement > 0  { duration.min((energy_available - energy_required_other_tasks) / resource_requirement)}
                                                else {duration};

                    if maximum_overlap >= (end_time - start_time + 1) || maximum_overlap >= duration { continue; }

                    // So the task cannot be scheduled in the range [start_time - duration + max_overlap + 1, end_time - max_overlap]
                    // We can either remove all values within the range, or we can only propagate the lower and upper bounds.
                    // Removing all values would introduce a runtime factor of O(number of possible timeslots) which can be very large.
                    // So let's only propogate the lower and upper bounds.
                    if task_start_time_ub <= end_time - maximum_overlap && start_time - duration + maximum_overlap < task_start_time_ub {
                        context.set_upper_bound(start_time_var, start_time - duration + maximum_overlap, conjunction!())?;
                    }
                    if task_start_time_lb > start_time - duration + maximum_overlap && end_time - maximum_overlap + 1 > task_start_time_lb {
                        context.set_lower_bound(start_time_var, end_time - maximum_overlap + 1, conjunction!())?;
                    }
                }    
            }
        }

        Ok(())
    }

    fn initialise_at_root(
        &mut self,
        context: &mut PropagatorInitialisationContext,
    ) -> Result<(), PropositionalConjunction> {
        for var in self.start_times.iter() {
            context.register(var.clone(), DomainEvents::ANY_INT);
        }

        // Conflict detection is handled in propagate
        Ok(())
    }
}
