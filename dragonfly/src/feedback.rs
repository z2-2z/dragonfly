use libafl::prelude::{
    Error,
    Event,
    EventFirer,
    ExitKind,
    Feedback,
    HasClientPerfMonitor,
    Named,
    ObserversTuple,
    UserStats,
    UsesInput,
};
use std::marker::PhantomData;

use crate::{
    graph::HasStateGraph,
    observer::StateObserver,
    stats,
};

const NAME_PREFIX: &str = "StateFeedbackFor";

#[derive(Debug)]
pub struct StateFeedback {
    name: String,
    observer_name: String,
}

impl StateFeedback {
    pub fn new(observer: &StateObserver) -> Self {
        Self {
            name: format!("{}{}", NAME_PREFIX, observer.name()),
            observer_name: observer.name().to_string(),
        }
    }
}

impl Named for StateFeedback {
    fn name(&self) -> &str {
        &self.name
    }
}

impl<S> Feedback<S> for StateFeedback
where
    S: UsesInput + HasClientPerfMonitor + HasStateGraph,
{
    fn is_interesting<EM, OT>(&mut self, state: &mut S, manager: &mut EM, _input: &<S as UsesInput>::Input, observers: &OT, _exit_kind: &ExitKind) -> Result<bool, Error>
    where
        EM: EventFirer<State = S>,
        OT: ObserversTuple<S>,
    {
        let state_observer = observers.match_name::<StateObserver>(&self.observer_name).ok_or_else(|| Error::empty_optional("StateFeedback could not find any StateObserver"))?;
        let interesting = state_observer.had_new_transitions();

        if interesting {
            let state_graph = state.get_stategraph()?;
            let graph_size = state_graph.edges().len();

            manager.fire(
                state,
                Event::UpdateUserStats {
                    name: stats::GRAPH_SIZE.to_string(),
                    value: UserStats::Number(graph_size as u64),
                    phantom: PhantomData,
                },
            )?;
        }

        Ok(interesting)
    }
}
