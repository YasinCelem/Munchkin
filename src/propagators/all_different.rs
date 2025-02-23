#![allow(unused, reason = "this file is a skeleton for the assignment")]

/*!
# Generative AI Usage

I used generative AI tools (e.g., ChatGPT) during the development of this file for:
1. **Outline of algorithms**:
   - To generate initial high‑level pseudocode and/or planning steps for the Ford–Fulkerson‑style maximum flow algorithm and the Tarjan SCC algorithm used in this file.
2. **Code refinement**:
   - To suggest improvements in function structure, variable naming, and documentation once the core logic was implemented.
*/

//! This module implements an all‑different constraint propagator using a bipartite
//! graph and a network flow formulation. The propagator builds an augmented graph
//! where variable nodes are connected to value nodes based on their domains. Using
//! a Ford–Fulkerson max‑flow algorithm and Tarjan’s SCC algorithm, inconsistent edges
//! (i.e. variable–value assignments that can be pruned) are identified and removed.

use crate::basic_types::PropagationStatusCP;
use crate::conjunction;
use crate::engine::cp::domain_events::DomainEvents;
use crate::engine::cp::propagation::propagation_context::ReadDomains;
use crate::engine::cp::propagation::PropagationContextMut;
use crate::engine::cp::propagation::Propagator;
use crate::engine::cp::propagation::PropagatorInitialisationContext;
use crate::predicates::PropositionalConjunction;
use crate::variables::IntegerVariable;
use std::collections::{BTreeSet, HashMap};

/// Represents a node in the augmented bipartite graph.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum NodeId {
    /// The source node of the flow network.
    Source,
    /// The sink node of the flow network.
    Sink,
    /// A variable node, identified by its index.
    Variable(i32),
    /// A value node, identified by the actual value from a variable's domain.
    Value(i32),
}

/// Represents a directed edge in the flow network.
#[derive(Clone, Debug)]
struct FlowEdge {
    /// The originating node.
    from: NodeId,
    /// The target node.
    to: NodeId,
    /// The capacity of the edge.
    capacity: i32,
    /// The current flow on the edge.
    flow: i32,
    /// Index of the reverse edge in the adjacency list of the `to` node.
    rev: usize,
}

impl FlowEdge {
    /// Creates a new flow edge with an initial flow of 0.
    ///
    /// # Arguments
    ///
    /// * `from` - The originating node.
    /// * `to` - The target node.
    /// * `capacity` - The maximum capacity for this edge.
    /// * `rev` - The index of the reverse edge.
    fn new(from: NodeId, to: NodeId, capacity: i32, rev: usize) -> Self {
        FlowEdge {
            from,
            to,
            capacity,
            flow: 0,
            rev,
        }
    }
}

/// Represents the augmented bipartite graph for the all‑different propagator.
/// It consists of variable nodes, value nodes (from the domains), and two special nodes:
/// the source and the sink.
struct BipartiteGraph<'a, Var: IntegerVariable> {
    /// A slice of the variables for which the propagator is built.
    variables: &'a [Var],
    /// Sorted list of distinct domain values across all variables.
    distinct_values: Vec<i32>,
    /// The flow network represented as an adjacency list.
    graph: HashMap<NodeId, Vec<FlowEdge>>,
}

impl<'a, Var: IntegerVariable> BipartiteGraph<'a, Var> {
    /// Builds the augmented bipartite graph from the provided variables and propagation context.
    ///
    /// This method collects all distinct domain values from the variables, creates nodes for
    /// each variable and value, and adds edges:
    ///
    /// 1. From the source to each variable node (capacity = 1).
    /// 2. From each variable node to every value in its domain (capacity = 1).
    /// 3. From each value node to the sink (capacity = 1).
    ///
    /// # Arguments
    ///
    /// * `context` - The propagation context providing domain information.
    /// * `variables` - A slice of variables under consideration.
    fn build(context: &PropagationContextMut, variables: &'a [Var]) -> Self {
        // Collect distinct domain values from all variables.
        let mut values_set = BTreeSet::new();
        for var in variables.iter() {
            let lb = context.lower_bound(var);
            let ub = context.upper_bound(var);
            for v in lb..=ub {
                if context.contains(var, v) {
                    let _ = values_set.insert(v);
                }
            }
        }
        let distinct_values: Vec<i32> = values_set.into_iter().collect();

        // Initialize the graph with source, sink, variable, and value nodes.
        let mut graph: HashMap<NodeId, Vec<FlowEdge>> = HashMap::new();
        let _ = graph.insert(NodeId::Source, Vec::new());
        let _ = graph.insert(NodeId::Sink, Vec::new());
        // Insert variable nodes (using their index as identifier).
        for (i, _var) in variables.iter().enumerate() {
            let _ = graph.insert(NodeId::Variable(i as i32), Vec::new());
        }
        // Insert value nodes (using the actual value).
        for &v in distinct_values.iter() {
            let _ = graph.insert(NodeId::Value(v), Vec::new());
        }

        /// Helper function to add an edge and its corresponding reverse edge to the graph.
        ///
        /// # Arguments
        ///
        /// * `graph` - The graph to which the edges are added.
        /// * `from` - The originating node.
        /// * `to` - The target node.
        /// * `capacity` - The capacity for the forward edge.
        fn add_edge(
            graph: &mut HashMap<NodeId, Vec<FlowEdge>>,
            from: NodeId,
            to: NodeId,
            capacity: i32,
        ) {
            let from_len = graph.get(&from).map(|v| v.len()).unwrap_or(0);
            let to_len = graph.get(&to).map(|v| v.len()).unwrap_or(0);
            // Forward edge.
            graph
                .get_mut(&from)
                .unwrap()
                .push(FlowEdge::new(from, to, capacity, to_len));
            // Reverse edge with 0 capacity.
            graph
                .get_mut(&to)
                .unwrap()
                .push(FlowEdge::new(to, from, 0, from_len));
        }

        // 1. Add edges from the source to each variable node.
        for (i, _var) in variables.iter().enumerate() {
            add_edge(&mut graph, NodeId::Source, NodeId::Variable(i as i32), 1);
        }

        // 2. For each variable, add an edge from its node to each value in its domain.
        for (i, var) in variables.iter().enumerate() {
            let var_index = i as i32;
            let lb = context.lower_bound(var);
            let ub = context.upper_bound(var);
            for v in lb..=ub {
                if context.contains(var, v) {
                    add_edge(&mut graph, NodeId::Variable(var_index), NodeId::Value(v), 1);
                }
            }
        }

        // 3. Add edges from each value node to the sink.
        for &v in distinct_values.iter() {
            add_edge(&mut graph, NodeId::Value(v), NodeId::Sink, 1);
        }

        Self {
            variables,
            distinct_values,
            graph,
        }
    }

    /// Finds an augmenting path from `source` to `sink` in the residual graph using DFS.
    ///
    /// Returns a vector of tuples `(u, edge_index)` that represents the path, where each
    /// tuple contains a node `u` and the index of the edge used from that node.
    ///
    /// # Arguments
    ///
    /// * `source` - The starting node of the search.
    /// * `sink` - The destination node of the search.
    fn find_augmenting_path(&self, source: NodeId, sink: NodeId) -> Option<Vec<(NodeId, usize)>> {
        let mut stack = Vec::new();
        let mut parent: HashMap<NodeId, (NodeId, usize)> = HashMap::new();

        // Start the DFS from the source node.
        let _ = stack.push(source);
        let _ = parent.insert(source, (source, 0)); // Dummy parent for source

        // Perform DFS until sink is reached or no more nodes are available.
        while let Some(u) = stack.pop() {
            if u == sink {
                break;
            }
            if let Some(edges) = self.graph.get(&u) {
                for (i, edge) in edges.iter().enumerate() {
                    let residual = edge.capacity - edge.flow;
                    if residual > 0 && !parent.contains_key(&edge.to) {
                        let _ = parent.insert(edge.to, (u, i));
                        let _ = stack.push(edge.to);
                    }
                }
            }
        }

        if !parent.contains_key(&sink) {
            return None;
        }

        // Reconstruct the path from sink to source.
        let mut path = Vec::new();
        let mut cur = sink;
        while cur != source {
            if let Some(&(prev, edge_idx)) = parent.get(&cur) {
                path.push((prev, edge_idx));
                cur = prev;
            } else {
                break;
            }
        }
        path.reverse();
        Some(path)
    }

    /// Computes the maximum flow in the bipartite graph using the Ford–Fulkerson method.
    ///
    /// Returns the total flow, which corresponds to the size of the maximum matching.
    fn ford_fulkerson(&mut self) -> i32 {
        let mut max_flow = 0;
        while let Some(path) = self.find_augmenting_path(NodeId::Source, NodeId::Sink) {
            // Determine the bottleneck capacity along the found path.
            let mut flow = i32::MAX;
            for &(u, edge_idx) in &path {
                let edge = &self.graph.get(&u).unwrap()[edge_idx];
                let residual = edge.capacity - edge.flow;
                flow = flow.min(residual);
            }
            // Augment the flow along the path.
            for &(u, edge_idx) in &path {
                // Update the forward edge.
                let (to, rev_index) = {
                    let edges = self.graph.get_mut(&u).unwrap();
                    let edge = &mut edges[edge_idx];
                    edge.flow += flow;
                    (edge.to, edge.rev)
                };
                // Update the reverse edge.
                {
                    let reverse_edges = self.graph.get_mut(&to).unwrap();
                    reverse_edges[rev_index].flow -= flow;
                }
            }
            max_flow += flow;
        }
        max_flow
    }

    /// Builds the residual graph (as an adjacency list) for variable and value nodes.
    ///
    /// Only nodes other than the source and sink are included.
    fn build_residual_adjacency(&self) -> HashMap<NodeId, Vec<NodeId>> {
        let mut residual_graph: HashMap<NodeId, Vec<NodeId>> = HashMap::new();

        // Initialize the residual graph for variable and value nodes.
        for &node in self.graph.keys() {
            if !matches!(node, NodeId::Source | NodeId::Sink) {
                let _ = residual_graph.insert(node, Vec::new());
            }
        }

        // Add edges that have residual capacity.
        for (&node, edges) in &self.graph {
            if matches!(node, NodeId::Source | NodeId::Sink) {
                continue;
            }
            for edge in edges {
                let residual_cap = edge.capacity - edge.flow;
                // Forward edge: include if residual capacity exists.
                if residual_cap > 0 && !matches!(edge.to, NodeId::Source | NodeId::Sink) {
                    residual_graph.get_mut(&node).unwrap().push(edge.to);
                }
                // Backward edge: include if there is flow to undo.
                if edge.flow > 0 && !matches!(edge.to, NodeId::Source | NodeId::Sink) {
                    residual_graph.get_mut(&edge.to).unwrap().push(node);
                }
            }
        }
        residual_graph
    }

    /// Finds all strongly connected components (SCCs) in the current residual graph using Tarjan’s algorithm.
    ///
    /// Returns a vector where each element is a list of nodes representing an SCC.
    fn find_sccs(&self) -> Vec<Vec<NodeId>> {
        let mut adjacency = self.build_residual_adjacency();

        let mut index_counter: i32 = 0;
        let mut index_map: HashMap<NodeId, i32> = HashMap::new();
        let mut lowlink_map: HashMap<NodeId, i32> = HashMap::new();
        let mut stack: Vec<NodeId> = Vec::new();
        let mut on_stack: HashMap<NodeId, bool> = HashMap::new();
        let mut sccs: Vec<Vec<NodeId>> = Vec::new();

        // Initialize Tarjan's data structures.
        for &node in adjacency.keys() {
            let _ = index_map.insert(node, -1);
            let _ = lowlink_map.insert(node, -1);
            let _ = on_stack.insert(node, false);
        }

        /// Inner function implementing Tarjan's strongconnect.
        fn strongconnect(
            v: NodeId,
            index_counter: &mut i32,
            index_map: &mut HashMap<NodeId, i32>,
            lowlink_map: &mut HashMap<NodeId, i32>,
            stack: &mut Vec<NodeId>,
            on_stack: &mut HashMap<NodeId, bool>,
            adjacency: &HashMap<NodeId, Vec<NodeId>>,
            sccs: &mut Vec<Vec<NodeId>>,
        ) {
            let _ = index_map.insert(v, *index_counter);
            let _ = lowlink_map.insert(v, *index_counter);
            *index_counter += 1;

            let _ = stack.push(v);
            let _ = on_stack.insert(v, true);

            if let Some(successors) = adjacency.get(&v) {
                for &w in successors {
                    if index_map[&w] == -1 {
                        strongconnect(
                            w,
                            index_counter,
                            index_map,
                            lowlink_map,
                            stack,
                            on_stack,
                            adjacency,
                            sccs,
                        );
                        let w_lowlink = lowlink_map[&w];
                        let v_lowlink = lowlink_map[&v];
                        let _ = lowlink_map.insert(v, std::cmp::min(v_lowlink, w_lowlink));
                    } else if on_stack[&w] {
                        let w_index = index_map[&w];
                        let v_lowlink = lowlink_map[&v];
                        let _ = lowlink_map.insert(v, std::cmp::min(v_lowlink, w_index));
                    }
                }
            }

            // If v is a root node, pop the stack and generate an SCC.
            if lowlink_map[&v] == index_map[&v] {
                let mut component = Vec::new();
                loop {
                    let w = stack.pop().unwrap();
                    let _ = on_stack.insert(w, false);
                    component.push(w);
                    if w == v {
                        break;
                    }
                }
                sccs.push(component);
            }
        }

        // Run Tarjan's algorithm on all unvisited nodes.
        for &node in adjacency.keys() {
            if index_map[&node] == -1 {
                strongconnect(
                    node,
                    &mut index_counter,
                    &mut index_map,
                    &mut lowlink_map,
                    &mut stack,
                    &mut on_stack,
                    &adjacency,
                    &mut sccs,
                );
            }
        }

        sccs
    }

    /// Identifies inconsistent edges between variable nodes and value nodes.
    ///
    /// An edge from a variable to a value is considered inconsistent and marked for pruning if:
    /// - The variable and value belong to different SCCs in the residual graph, and
    /// - The SCC corresponding to the value contains more than one node.
    ///
    /// Returns a vector of `(variable_index, value)` pairs that should be pruned.
    fn find_inconsistent_edges(&self) -> Vec<(i32, i32)> {
        // 1. Compute SCCs.
        let sccs = self.find_sccs();

        // Map each SCC index to its size.
        let mut scc_size: HashMap<usize, usize> = HashMap::new();
        for (scc_index, scc) in sccs.iter().enumerate() {
            let _ = scc_size.insert(scc_index, scc.len());
        }

        // Map each node to its SCC index.
        let mut node_to_scc: HashMap<NodeId, usize> = HashMap::new();
        for (scc_index, scc) in sccs.iter().enumerate() {
            for &node in scc {
                let _ = node_to_scc.insert(node, scc_index);
            }
        }

        let mut inconsistent_edges = Vec::new();
        // 2. Iterate over edges from variable nodes to value nodes.
        for (&node, edges) in &self.graph {
            if let NodeId::Variable(_) = node {
                for edge in edges {
                    if let (NodeId::Variable(var_index), NodeId::Value(val)) = (edge.from, edge.to)
                    {
                        if let (Some(&scc_from), Some(&scc_to)) =
                            (node_to_scc.get(&edge.from), node_to_scc.get(&edge.to))
                        {
                            // Mark for pruning if in different SCCs and the value SCC has size > 1.
                            if scc_from != scc_to {
                                let size_to = scc_size.get(&scc_to).copied().unwrap_or(0);
                                if size_to > 1 {
                                    inconsistent_edges.push((var_index, val));
                                }
                            }
                        }
                    }
                }
            }
        }
        inconsistent_edges
    }
}

pub(crate) struct AllDifferentPropagator<Var> {
    variables: Box<[Var]>, // TODO: you can add more fields here!
}

impl<Var> AllDifferentPropagator<Var> {
    pub(crate) fn new(variables: Box<[Var]>) -> Self {
        Self { variables }
    }
}

impl<Var: IntegerVariable + 'static> Propagator for AllDifferentPropagator<Var> {
    fn name(&self) -> &str {
        "AllDifferent"
    }

    /// Propagates the all‑different constraint by building the bipartite graph,
    /// computing the maximum flow, identifying SCCs in the residual graph, and
    /// pruning inconsistent variable–value assignments.
    fn propagate(&self, mut context: PropagationContextMut) -> PropagationStatusCP {
        let mut graph = BipartiteGraph::build(&context, &self.variables);
        let max_flow = graph.ford_fulkerson();

        // Identify inconsistent edges that cross SCC boundaries.
        let scc_list = graph.find_sccs();
        let inconsistent = graph.find_inconsistent_edges();

        for (var, value) in inconsistent {
            context.remove(&self.variables[var as usize], value, conjunction!())?;
        }

        Ok(())
    }

    fn initialise_at_root(
        &mut self,
        context: &mut PropagatorInitialisationContext,
    ) -> Result<(), PropositionalConjunction> {
        for var in self.variables.iter() {
            context.register(var.clone(), DomainEvents::ANY_INT);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::test_helper::TestSolver;

    /// Tests the AllDifferent propagator by creating a simple constraint problem
    /// based on the lecture notes, and asserting that inconsistent domain values
    /// are pruned.
    #[test]
    fn test_propagator() {
        let mut solver = TestSolver::default();

        let x1 = solver.new_variable(0, 1);
        let x2 = solver.new_variable(0, 2);
        let x3 = solver.new_variable(2, 3);
        let x4 = solver.new_variable(2, 4);
        let x5 = solver.new_variable(2, 4);

        let variables = Box::new([x1, x2, x3, x4, x5]);

        let _ = solver
            .new_propagator(AllDifferentPropagator::new(variables))
            .expect("Expected no error");

        solver.assert_bounds(x2, 0, 1);
    }
}
