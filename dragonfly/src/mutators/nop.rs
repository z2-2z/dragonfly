use libafl::prelude::{
    Error,
    Input,
    MutationResult,
    Mutator,
    Named,
};

pub struct NopMutator;

impl NopMutator {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {}
    }
}

impl<I, S> Mutator<I, S> for NopMutator
where
    I: Input,
{
    fn mutate(&mut self, _state: &mut S, _input: &mut I, _stage_idx: i32) -> Result<MutationResult, Error> {
        Ok(MutationResult::Mutated)
    }
}

impl Named for NopMutator {
    fn name(&self) -> &str {
        "NopMutator"
    }
}
