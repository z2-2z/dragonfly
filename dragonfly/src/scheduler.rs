use libafl::prelude::{
    Scheduler, UsesState, HasCorpus, UsesInput, Error,
    ObserversTuple, CorpusId, Named, Corpus,
    minimizer::IsFavoredMetadata, HasMetadata,
};
use ahash::{AHashMap, AHashSet};
use crate::observer::{StateObserver, State};
use itertools::Itertools;

const FAVORITE_STATES_COUNT: usize = 1;

#[derive(Default)]
struct StateMetadata {
    n_fuzz: usize,
    n_selected: usize,
    seeds: AHashSet<CorpusId>,
}

impl StateMetadata {
    fn score(&self) -> usize {
        let capped_n_fuzz = 1.0 + (1.0 + self.n_fuzz as f64).log2();
        let capped_n_selected = 1.0 + (1.0 + self.n_selected as f64).log10();
        (2.0f64.powf(capped_n_fuzz) * (capped_n_selected)).ceil() as usize
    }
}

pub struct StateScheduler<CS> {
    base: CS,
    observer_name: String,
    current_sequence: Vec<State>,
    metadata: AHashMap<State, StateMetadata>,
    favorites: AHashSet<CorpusId>,
}

impl<CS> UsesState for StateScheduler<CS>
where
    CS: UsesState,
{
    type State = CS::State;
}

impl<CS> StateScheduler<CS>
where
    CS: Scheduler,
    CS::State: HasCorpus,
{
    pub fn new(base: CS, state_observer: &StateObserver) -> Self {
        Self {
            base,
            observer_name: state_observer.name().to_string(),
            current_sequence: Vec::with_capacity(256),
            metadata: AHashMap::new(),
            favorites: AHashSet::new(),
        }
    }
    
    fn update_n_fuzz(&mut self, state: State) {
        let meta = self.metadata.entry(state).or_insert_with(StateMetadata::default);
        meta.n_fuzz = meta.n_fuzz.saturating_add(1);
    }
    
    fn update_n_selected(&mut self, seed: CorpusId) {
        for state in &self.current_sequence {
            let meta = self.metadata.get_mut(state).unwrap();
            
            if meta.seeds.contains(&seed) {
                meta.n_selected = meta.n_selected.saturating_add(1);
            }
        }
    }
    
    fn add_seed(&mut self, seed: CorpusId) {
        for state in &self.current_sequence {
            let meta = self.metadata.entry(*state).or_insert_with(StateMetadata::default);
            meta.seeds.insert(seed);
        }
    }
    
    fn get_least_fuzzed_states(&self, result: &mut [State; FAVORITE_STATES_COUNT]) -> usize {
        let mut ret = 0;
        
        for (key, _) in self.metadata.iter().sorted_by(|(_, l), (_, r)| l.score().cmp(&r.score())).take(result.len()) {
            result[ret].copy_from_slice(key);
            ret += 1;
        }
        
        ret
    }
}

impl<CS> Scheduler for StateScheduler<CS>
where
    CS: Scheduler,
    CS::State: HasCorpus,
{
    fn on_evaluation<OT>(&mut self, fuzzer_state: &mut Self::State, input: &<Self::State as UsesInput>::Input, observers: &OT) -> Result<(), Error>
    where
        OT: ObserversTuple<Self::State>,
    {
        let state_observer = observers.match_name::<StateObserver>(&self.observer_name).expect("no state observer found");
        let states = state_observer.get_all_states();
        let states_len = states.len();
        
        println!("on_evaluation: total_states = {}", states_len);
        
        self.current_sequence.resize(states_len, State::default());
        self.current_sequence[0..states_len].copy_from_slice(states);
        
        for state in states {
            self.update_n_fuzz(*state);
        }
        
        self.base.on_evaluation(fuzzer_state, input, observers)
    }
    
    fn on_add(&mut self, fuzzer_state: &mut Self::State, idx: CorpusId) -> Result<(), Error> {
        self.add_seed(idx);
        self.base.on_add(fuzzer_state, idx)
    }

    fn next(&mut self, fuzzer_state: &mut Self::State) -> Result<CorpusId, Error> {
        /* clear favorites from previous run */
        for id in self.favorites.drain() {
            let _ = fuzzer_state.corpus_mut().get(id)?.borrow_mut().metadata_map_mut().remove::<IsFavoredMetadata>();
        }
        
        /* Select up to FAVORITE_STATES_COUNT states */
        let mut selected_states = [State::default(); FAVORITE_STATES_COUNT];
        let selected_states_size = self.get_least_fuzzed_states(&mut selected_states);
        
        for state in &selected_states[..selected_states_size] {
            for seed in &self.metadata.get(state).unwrap().seeds {
                fuzzer_state.corpus_mut().get(*seed)?.borrow_mut().add_metadata::<IsFavoredMetadata>(IsFavoredMetadata {});
                self.favorites.insert(*seed);
                println!("next: mark {} as favorite", seed);
            }
        }
        
        self.base.next(fuzzer_state)
    }
    
    fn set_current_scheduled(&mut self, state: &mut Self::State, next_id: Option<CorpusId>) -> Result<(), Error> {
        if let Some(id) = next_id {
            println!("set_current_scheduled: Selecting {}", id);
            self.update_n_selected(id);
        }
        
        self.base.set_current_scheduled(state, next_id)
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
        }.score()
    }
    
    #[test]
    fn print_score() {
        assert!(score(2, 3) < score(2, 4));
        assert!(score(99999999, 8888888) < score(99999999, 8888888 + 1));
        assert!(score(3, 0) < score(2, 64));
        assert!(score(4, 128) < score(128, 4));
    }
}
