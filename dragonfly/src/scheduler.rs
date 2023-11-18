use crate::observer::{
    State,
    StateObserver,
};
use ahash::{
    AHashMap,
    AHashSet,
};
use libafl_bolts::{
    impl_serdeany,
    Named,
    rands::Rand,
};
use libafl::prelude::{
    minimizer::IsFavoredMetadata,
    powersched::PowerSchedule,
    Corpus,
    CorpusId,
    Error,
    HasCorpus,
    HasMetadata,
    HasRand,
    HasTestcase,
    MapObserver,
    ObserversTuple,
    Scheduler,
    Testcase,
    TestcaseScore,
    UsesInput,
    UsesState,
    WeightedScheduler,
};
use serde::{
    Deserialize,
    Serialize,
};
use std::marker::PhantomData;

#[derive(Debug, Serialize, Deserialize)]
pub struct ReachesFavoredStateMetadata;

impl_serdeany!(ReachesFavoredStateMetadata);

const MAX_FAVORITE_COUNT: usize = 8;

type Score = usize;

#[derive(Default)]
struct StateMetadata {
    n_fuzz: usize,
    n_selected: usize,
    seeds: AHashSet<CorpusId>,
}

impl StateMetadata {
    fn score(&self) -> Score {
        let capped_n_fuzz = 1.0 + (1.0 + self.n_fuzz as f64).log2();
        let capped_n_selected = 1.0 + (1.0 + self.n_selected as f64).log10();
        (2.0f64.powf(capped_n_fuzz) * (capped_n_selected)).ceil() as usize
    }
}

pub struct StateAwareWeightedScheduler<O, S>
where
    O: MapObserver,
    S: HasCorpus + HasMetadata + HasRand + HasTestcase,
{
    base: WeightedScheduler<FavoredStateTestcaseWeight<S>, O, S>,
    observer_name: String,
    current_sequence: Vec<State>,
    metadata: AHashMap<State, StateMetadata>,
    favorites: AHashSet<CorpusId>,
}

impl<O, S> UsesState for StateAwareWeightedScheduler<O, S>
where
    O: MapObserver,
    S: HasCorpus + HasMetadata + HasRand + HasTestcase,
{
    type State = S;
}

impl<O, S> StateAwareWeightedScheduler<O, S>
where
    O: MapObserver,
    S: HasCorpus + HasMetadata + HasRand + HasTestcase,
{
    pub fn new(state: &mut S, map_observer: &O, strat: Option<PowerSchedule>, state_observer: &StateObserver) -> Self {
        Self {
            base: WeightedScheduler::with_schedule(state, map_observer, strat),
            observer_name: state_observer.name().to_string(),
            current_sequence: Vec::with_capacity(256),
            metadata: AHashMap::new(),
            favorites: AHashSet::new(),
        }
    }

    fn update_n_fuzz(&mut self) {
        for state in &self.current_sequence {
            let meta = self.metadata.entry(*state).or_insert_with(StateMetadata::default);
            meta.n_fuzz = meta.n_fuzz.saturating_add(1);
        }
    }

    fn update_seeds(&mut self, seed: CorpusId) {
        for state in &self.current_sequence {
            let meta = self.metadata.entry(*state).or_insert_with(StateMetadata::default);
            meta.seeds.insert(seed);
        }
    }

    fn get_least_fuzzed_state(&self) -> Option<State> {
        let mut ret = None;
        let mut current_score = Score::MAX;

        for (state, metadata) in self.metadata.iter() {
            let score = metadata.score();

            if score < current_score {
                current_score = score;
                ret = Some(state);
            }
        }

        ret.copied()
    }

    fn calculate_favorites(&mut self, fuzzer_state: &mut S) -> Result<(), Error> {
        if let Some(state) = self.get_least_fuzzed_state() {
            let meta = self.metadata.get_mut(&state).unwrap();
            let mut favorites = AHashSet::with_capacity(MAX_FAVORITE_COUNT);

            let skip_limit = meta.seeds.len().saturating_sub(MAX_FAVORITE_COUNT);
            let skip_amount = fuzzer_state.rand_mut().below(skip_limit as u64 + 1) as usize;

            for seed in meta.seeds.iter().skip(skip_amount).take(MAX_FAVORITE_COUNT) {
                fuzzer_state.corpus_mut().get(*seed)?.borrow_mut().add_metadata::<ReachesFavoredStateMetadata>(ReachesFavoredStateMetadata {});
                favorites.insert(*seed);

                #[cfg(test)]
                println!("mark {} as favorite", seed);
            }

            for id in self.favorites.difference(&favorites) {
                drop(fuzzer_state.corpus_mut().get(*id)?.borrow_mut().metadata_map_mut().remove::<ReachesFavoredStateMetadata>());
            }

            self.favorites = favorites;
            meta.n_selected = meta.n_selected.saturating_add(1);
        }

        Ok(())
    }
}

impl<O, S> Scheduler for StateAwareWeightedScheduler<O, S>
where
    O: MapObserver,
    S: HasCorpus + HasMetadata + HasRand + HasTestcase,
{
    fn on_evaluation<OT>(&mut self, fuzzer_state: &mut S, input: &<S as UsesInput>::Input, observers: &OT) -> Result<(), Error>
    where
        OT: ObserversTuple<S>,
    {
        let state_observer = observers.match_name::<StateObserver>(&self.observer_name).expect("no state observer found");
        let states = state_observer.get_all_states();
        let states_len = states.len();
        self.current_sequence.resize(states_len, State::default());
        self.current_sequence[0..states_len].copy_from_slice(states);
        self.update_n_fuzz();
        self.base.on_evaluation(fuzzer_state, input, observers)
    }

    fn on_add(&mut self, fuzzer_state: &mut Self::State, idx: CorpusId) -> Result<(), Error> {
        self.update_seeds(idx);
        self.calculate_favorites(fuzzer_state)?;
        self.base.on_add(fuzzer_state, idx)
    }

    fn next(&mut self, fuzzer_state: &mut Self::State) -> Result<CorpusId, Error> {
        self.base.next(fuzzer_state)
    }

    fn set_current_scheduled(&mut self, state: &mut Self::State, next_id: Option<CorpusId>) -> Result<(), Error> {
        self.base.set_current_scheduled(state, next_id)
    }
}

#[derive(Debug, Clone)]
struct FavoredStateTestcaseWeight<S> {
    phantom: PhantomData<S>,
}

impl<S> TestcaseScore<S> for FavoredStateTestcaseWeight<S>
where
    S: HasCorpus + HasMetadata,
{
    fn compute(_state: &S, entry: &mut Testcase<<S>::Input>) -> Result<f64, Error> {
        let mut result = 1.0;

        if entry.has_metadata::<IsFavoredMetadata>() {
            result *= 2.0;
        }

        if entry.has_metadata::<ReachesFavoredStateMetadata>() {
            result *= 2.0;
        }

        Ok(result)
    }
}

#[cfg(test)]
mod metadata_tests {
    use super::*;

    fn score(n_fuzz: usize, n_selected: usize) -> usize {
        StateMetadata {
            n_fuzz,
            n_selected,
            seeds: AHashSet::new(),
        }
        .score()
    }

    #[test]
    fn print_score() {
        assert!(score(2, 3) < score(2, 4));
        assert!(score(99999999, 8888888) < score(99999999, 8888888 + 1));
        assert!(score(3, 0) < score(2, 64));
        assert!(score(4, 128) < score(128, 4));
    }
}
