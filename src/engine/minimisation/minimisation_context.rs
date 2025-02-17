use crate::engine::cp::propagation::propagation_context::HasAssignments;
use crate::engine::cp::AssignmentsInteger;
use crate::engine::cp::VariableLiteralMappings;
use crate::engine::sat::AssignmentsPropositional;
use crate::predicates::IntegerPredicate;
use crate::variables::Literal;

#[derive(Debug, Clone, Copy)]
pub(crate) struct MinimisationContext<'a> {
    assignments_integer: &'a AssignmentsInteger,
    assignments_propositional: &'a AssignmentsPropositional,
    #[allow(unused, reason = "will be used in the assignments")]
    variable_literal_mappings: &'a VariableLiteralMappings,
}

impl<'a> MinimisationContext<'a> {
    pub(crate) fn new(
        assignments_integer: &'a AssignmentsInteger,
        assignments_propositional: &'a AssignmentsPropositional,
        variable_literal_mappings: &'a VariableLiteralMappings,
    ) -> Self {
        Self {
            assignments_integer,
            assignments_propositional,
            variable_literal_mappings,
        }
    }

    #[allow(unused, reason = "will be used in the assignments")]
    pub(crate) fn get_predicates_for_literal(
        &self,
        literal: Literal,
    ) -> impl Iterator<Item = IntegerPredicate> + '_ {
        self.variable_literal_mappings
            .get_predicates_for_literal(literal)
    }
}

impl HasAssignments for MinimisationContext<'_> {
    fn assignments_integer(&self) -> &AssignmentsInteger {
        self.assignments_integer
    }

    fn assignments_propositional(&self) -> &AssignmentsPropositional {
        self.assignments_propositional
    }
}
