use crate::executable::SelectionSet;
use crate::{Indexable, VariableDirectives};

pub trait FragmentDefinition: Indexable {
    type Directives: VariableDirectives;
    type SelectionSet: SelectionSet;

    fn name(&self) -> &str;
    fn type_condition(&self) -> &str;
    fn directives(&self) -> &Self::Directives;
    fn selection_set(&self) -> &Self::SelectionSet;
}
