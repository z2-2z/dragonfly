use crate::{
    prelude::*,
    tt::*,
};
use libafl::prelude::{
    current_nanos,
    tuple_list,
    CrashFeedback,
    Error,
    Evaluator,
    ExitKind,
    Fuzzer,
    HasMetadata,
    InMemoryCorpus,
    InProcessExecutor,
    NopMonitor,
    RandScheduler,
    SimpleEventManager,
    StdFuzzer,
    StdMutationalStage,
    StdRand,
    StdState,
    TimeFeedback,
    TimeObserver,
    Tokens,
    StdScheduledMutator,
};

#[test]
fn main() -> Result<(), Error> {
    let seed = current_nanos();

    let monitor = NopMonitor::new();
    let mut mgr = SimpleEventManager::new(monitor);

    let time_observer = TimeObserver::new("time");

    let mut feedback = TimeFeedback::with_observer(&time_observer);

    let mut objective = CrashFeedback::new();

    let mut state = StdState::new(StdRand::with_seed(seed), InMemoryCorpus::<DragonflyInput<TokenStream>>::new(), InMemoryCorpus::new(), &mut feedback, &mut objective).unwrap();
    state.init_stategraph();

    let mut tokens = Tokens::new();
    tokens.add_token(&b"TOKEN1-A".to_vec());
    tokens.add_token(&b"TOKEN2-B".to_vec());
    tokens.add_token(&b"TOKEN3-C".to_vec());
    tokens.add_token(&b"TOKEN4-D".to_vec());
    tokens.add_token(&b"TOKEN5-E".to_vec());
    state.add_metadata(tokens);

    let max_packets = 16;
    let packet_mutations = PacketSelectorMutator::new(StdScheduledMutator::with_max_stack_pow(
        tuple_list!(
            PacketCrossoverInsertMutator::new(),
            PacketCrossoverReplaceMutator::new(),
            PacketDeleteMutator::new(0),
            PacketDuplicateMutator::new(max_packets),
            InsertRandomPacketMutator::new(),
            PacketReorderMutator::new(),
            PacketRepeatMutator::new(max_packets, max_packets)
        ),
        16,
    ));

    let mutational = StdMutationalStage::new(packet_mutations);

    let scheduler = RandScheduler::new();

    let mut fuzzer = StdFuzzer::new(scheduler, feedback, objective);

    let mut harness = |_: &_| ExitKind::Ok;
    let mut executor = InProcessExecutor::new(&mut harness, tuple_list!(time_observer), &mut fuzzer, &mut state, &mut mgr)?;

    let mut stages = tuple_list!(mutational);

    let input = DragonflyInput::new(vec![TokenStream::builder().constant("").whitespace("").text("").blob("").build(), TokenStream::builder().build()]);
    fuzzer.add_input(&mut state, &mut executor, &mut mgr, input)?;

    fuzzer.fuzz_loop(&mut stages, &mut executor, &mut state, &mut mgr)?;

    Ok(())
}
