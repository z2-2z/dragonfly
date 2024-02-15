use std::{path::PathBuf, ptr::write};
use libafl::monitors::SimpleMonitor;
use libafl::{
    corpus::{InMemoryCorpus, OnDiskCorpus},
    events::SimpleEventManager,
    executors::{inprocess::InProcessExecutor, ExitKind},
    feedbacks::{CrashFeedback, MaxMapFeedback},
    fuzzer::{Fuzzer, StdFuzzer},
    generators::RandPrintablesGenerator,
    inputs::{BytesInput, HasTargetBytes},
    mutators::scheduled::{havoc_mutations, StdScheduledMutator},
    observers::StdMapObserver,
    schedulers::QueueScheduler,
    stages::mutational::StdMutationalStage,
    state::StdState,
    prelude::{Evaluator, Tokens, HasMetadata},
};
use libafl_bolts::{current_nanos, rands::StdRand, tuples::tuple_list, AsSlice};
use crate::{components::{DragonflyInput, PacketCopyMutator, PacketDeleteMutator, PacketRepeatMutator, PacketSwapMutator, TokenStreamMutator, PacketContentMutator}, tokens::TokenStream};

static mut SIGNALS: [u8; 16] = [0; 16];
static mut SIGNALS_PTR: *mut u8 = unsafe { SIGNALS.as_mut_ptr() };

#[test]
fn simple_harness() {
    let mut serbuf = vec![0; 4096];
    
    // The closure that we want to fuzz
    let mut harness = |input: &DragonflyInput<TokenStream>| {
        //println!("{:?}", input);
        input.serialize_dragonfly_format(&mut serbuf);
        ExitKind::Ok
    };

    // Create an observation channel using the signals map
    let observer = unsafe { StdMapObserver::from_mut_ptr("signals", SIGNALS_PTR, SIGNALS.len()) };

    // Feedback to rate the interestingness of an input
    let mut feedback = MaxMapFeedback::new(&observer);

    // A feedback to choose if an input is a solution or not
    let mut objective = CrashFeedback::new();

    // create a State from scratch
    let mut state = StdState::new(
        // RNG
        StdRand::with_seed(current_nanos()),
        // Corpus that will be evolved, we keep it in memory for performance
        InMemoryCorpus::new(),
        // Corpus in which we store solutions (crashes in this example),
        // on disk so the user can get them after stopping the fuzzer
        InMemoryCorpus::new(),
        // States of the feedbacks.
        // The feedbacks can report the data that should persist in the State.
        &mut feedback,
        // Same for objective feedbacks
        &mut objective,
    )
    .unwrap();

    let mut dict = Tokens::new();
    assert!( dict.add_token(&b"TOKEN".to_vec()) );
    state.add_metadata(dict);

    let mon = SimpleMonitor::new(|s| println!("{s}"));

    // The event manager handle the various events generated during the fuzzing loop
    // such as the notification of the addition of a new item to the corpus
    let mut mgr = SimpleEventManager::new(mon);

    // A queue policy to get testcasess from the corpus
    let scheduler = QueueScheduler::new();

    // A fuzzer with feedbacks and a corpus scheduler
    let mut fuzzer = StdFuzzer::new(scheduler, feedback, objective);

    // Create the executor for an in-process function with just one observer
    let mut executor = InProcessExecutor::new(
        &mut harness,
        tuple_list!(observer),
        &mut fuzzer,
        &mut state,
        &mut mgr,
    )
    .expect("Failed to create the Executor");

    let input = DragonflyInput::new(vec![
        "hello world".parse::<TokenStream>().unwrap(),
    ]);
    fuzzer.add_input(&mut state, &mut executor, &mut mgr, input).unwrap();

    // Setup a mutational stage with a basic bytes mutator
    let max_packets = 16;
    let mutators = tuple_list!(
        PacketCopyMutator::new(max_packets),
        PacketDeleteMutator::new(0),
        PacketRepeatMutator::new(max_packets),
        PacketSwapMutator::new(),
        PacketContentMutator::new(TokenStreamMutator::new(128))
    );
    let mutator = StdScheduledMutator::with_max_stack_pow(mutators, 2);
    let mut stages = tuple_list!(StdMutationalStage::new(mutator));

    fuzzer
        .fuzz_loop(&mut stages, &mut executor, &mut state, &mut mgr)
        .expect("Error in the fuzzing loop");
}
