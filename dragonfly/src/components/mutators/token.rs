use libafl_bolts::prelude::{Rand, StdRand};
use libafl::prelude::{MutationResult, Error, HasRand, HasMetadata, Tokens, HasCorpus, random_corpus_id, Corpus, UsesInput};
use std::hash::Hash;
use crate::{
    components::{PacketMutator, Packet, DragonflyInput},
    tokens::{HasTokenStream, mutators::*},
};
use serde::{Serialize, Deserialize};

const STACKS: [usize; 4] = [
    2,
    4,
    16,
    128,
];

pub struct TokenStreamMutator {
    max_tokens: usize,
    rand: StdRand,
}

impl TokenStreamMutator {
    pub fn new(max_tokens: usize) -> Self {
        Self {
            max_tokens,
            rand: StdRand::with_seed(0),
        }
    }
}

impl<P, S> PacketMutator<P, S> for TokenStreamMutator
where
    P: Packet + HasTokenStream + std::fmt::Debug + Clone + Hash + Serialize + for<'a> Deserialize<'a>,
    S: HasRand + HasMetadata + HasCorpus,
    S: UsesInput<Input = DragonflyInput<P>>,
{
    fn mutate_packet(&mut self, state: &mut S, packet: &mut P) -> Result<MutationResult, Error> {
        if !packet.has_token_stream() {
            return Ok(MutationResult::Skipped);
        }
        
        let stream = packet.token_stream_mut();
        let stack = state.rand_mut().choose(STACKS);
        let mut mutated = false;
        let mut num_mutations = 19;
        
        self.rand.set_seed(state.rand_mut().next());
        
        let dict = state.metadata_map().get::<Tokens>();
        
        if dict.is_none() {
            num_mutations -= 3;
        }
        
        for _ in 0..stack {
            mutated |= match self.rand.below(num_mutations) {
                0 => mutate_copy(&mut self.rand, stream, self.max_tokens),
                1 => {
                    let idx = random_corpus_id!(state.corpus(), &mut self.rand);
                    
                    if state.corpus().current().as_ref() == Some(&idx) {
                        continue;
                    }
                    
                    let mut other_testcase = state.corpus().get(idx)?.borrow_mut();
                    let other_testcase = other_testcase.load_input(state.corpus())?;
                    
                    if other_testcase.packets().is_empty() {
                        continue;
                    }
                    
                    let idx = self.rand.below(other_testcase.packets().len() as u64) as usize;
                    let other_packet = &other_testcase.packets()[idx];
                    
                    if !other_packet.has_token_stream() {
                        continue;
                    }
                    
                    mutate_crossover_insert(&mut self.rand, stream, other_packet.token_stream(), self.max_tokens)
                },
                2 => {
                    let idx = random_corpus_id!(state.corpus(), &mut self.rand);
                    
                    if state.corpus().current().as_ref() == Some(&idx) {
                        continue;
                    }
                    
                    let mut other_testcase = state.corpus().get(idx)?.borrow_mut();
                    let other_testcase = other_testcase.load_input(state.corpus())?;
                    
                    if other_testcase.packets().is_empty() {
                        continue;
                    }
                    
                    let idx = self.rand.below(other_testcase.packets().len() as u64) as usize;
                    let other_packet = &other_testcase.packets()[idx];
                    
                    if !other_packet.has_token_stream() {
                        continue;
                    }
                    
                    mutate_crossover_replace(&mut self.rand, stream, other_packet.token_stream(), self.max_tokens)
                },
                3 => mutate_delete(&mut self.rand, stream),
                4 => mutate_flip(&mut self.rand, stream),
                5 => mutate_interesting(&mut self.rand, stream),
                6 => mutate_random_insert(&mut self.rand, stream, self.max_tokens),
                7 => mutate_random_replace(&mut self.rand, stream),
                8 => mutate_repeat_char::<_, 4096>(&mut self.rand, stream),
                9 => mutate_repeat_token::<_, 4096>(&mut self.rand, stream, self.max_tokens),
                10 => mutate_special_insert(&mut self.rand, stream),
                11 => mutate_special_replace(&mut self.rand, stream),
                12 => mutate_split(&mut self.rand, stream, self.max_tokens),
                13 => mutate_swap_tokens(&mut self.rand, stream),
                14 => mutate_swap_words(&mut self.rand, stream),
                15 => mutate_truncate(&mut self.rand, stream),
                16 => {
                    debug_assert!(dict.is_some());
                    let dict = unsafe { dict.unwrap_unchecked() };
                    mutate_dict_insert(&mut self.rand, stream, dict, self.max_tokens)
                },
                17 => {
                    debug_assert!(dict.is_some());
                    let dict = unsafe { dict.unwrap_unchecked() };
                    mutate_dict_replace(&mut self.rand, stream, dict)
                },
                18 => {
                    debug_assert!(dict.is_some());
                    let dict = unsafe { dict.unwrap_unchecked() };
                    mutate_swap_constants(&mut self.rand, stream, dict)
                },
                _ => unreachable!(),
            };
        }
        
        if mutated {
            Ok(MutationResult::Mutated)
        } else {
            Ok(MutationResult::Skipped)
        }
    }
}
