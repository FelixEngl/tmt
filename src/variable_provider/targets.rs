use crate::variable_provider::VariableProviderError;

trait Sealed{}

#[allow(private_bounds)]
pub trait Target: Sealed {
    fn make_error(id: usize, id_max: usize) -> VariableProviderError;
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Topics;

impl Sealed for Topics {}

impl Target for Topics {
    fn make_error(id: usize, id_max: usize) -> VariableProviderError {
        VariableProviderError::TopicNotFound {
            topic_id: id,
            topic_count: id_max
        }
    }
}

#[derive(Debug, Default, Copy, Clone)]
pub struct Words;

impl Sealed for Words {}

impl Target for Words {
    fn make_error(id: usize, id_max: usize) -> VariableProviderError {
        VariableProviderError::WordNotFound {
            word_id: id,
            word_count: id_max
        }
    }
}
