use crate::{
    prelude::*,
    tt::*,
};
use libafl_bolts::prelude::{
    current_nanos,
    tuple_list,
    StdRand,
};
use libafl::prelude::{
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
    StdState,
    TimeFeedback,
    TimeObserver,
    Tokens,
};

fn test_token_stream(input: &DragonflyInput<TokenStream>) -> ExitKind {
    for packet in input.packets() {
        let mut builder = TokenStream::builder();

        for token in packet.tokens() {
            println!("token = {:?}", token);

            match token {
                TextToken::Constant(data) => builder = builder.constant(data),
                TextToken::Number(data) => builder = builder.number(std::str::from_utf8(data).unwrap()),
                TextToken::Whitespace(data) => builder = builder.whitespace(std::str::from_utf8(data).unwrap()),
                TextToken::Text(data) => builder = builder.text(std::str::from_utf8(data).unwrap()),
                TextToken::Blob(data) => builder = builder.blob(data),
            }
        }
    }

    ExitKind::Ok
}

#[test]
fn test_roundtrip() -> Result<(), Error> {
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

    let max_tokens = 16;
    let tt_mutations = PacketSelectorMutator::new(ScheduledPacketMutator::with_max_stack_pow(
        tuple_list!(
            NopPacketMutator::new(),
            TokenStreamInsertRandomMutator::new(max_tokens),
            TokenReplaceRandomMutator::new(),
            TokenSplitMutator::new(max_tokens),
            TokenStreamInsertInterestingMutator::new(max_tokens),
            TokenReplaceInterestingMutator::new(),
            TokenValueCopyMutator::new(),
            TokenValueInsertRandomMutator::new(),
            TokenStreamCopyMutator::new(max_tokens),
            TokenStreamSwapMutator::new(),
            TokenStreamDeleteMutator::new(0),
            TokenRotateCharMutator::new(),
            TokenValueDeleteMutator::new(0),
            TokenInsertSpecialCharMutator::new(),
            TokenStreamDictInsertMutator::new(max_tokens),
            TokenReplaceDictMutator::new(),
            TokenStreamScannerMutator::new(max_tokens),
        ),
        16,
    ));

    let mutational = StdMutationalStage::new(tt_mutations);

    let scheduler = RandScheduler::new();

    let mut fuzzer = StdFuzzer::new(scheduler, feedback, objective);

    let mut harness = test_token_stream;
    let mut executor = InProcessExecutor::new(&mut harness, tuple_list!(time_observer), &mut fuzzer, &mut state, &mut mgr)?;

    let mut stages = tuple_list!(mutational);

    let input = DragonflyInput::new(vec![TokenStream::builder().constant("").whitespace("").text("").blob("").build(), TokenStream::builder().build()]);
    fuzzer.add_input(&mut state, &mut executor, &mut mgr, input)?;

    fuzzer.fuzz_loop(&mut stages, &mut executor, &mut state, &mut mgr)?;

    Ok(())
}

fn print_token_stream(input: &DragonflyInput<TokenStream>) -> ExitKind {
    assert_eq!(input.packets().len(), 1);
    
    let mut buf = [0; 4096];
    
    for packet in input.packets() {
        let size = packet.generate_text(&mut buf);
        println!("NEW MUTATION: {}", String::from_utf8_lossy(&buf[..size]));
    }

    ExitKind::Ok
}

#[test]
fn test_outcome() -> Result<(), Error> {
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

    let max_tokens = 16;
    let tt_mutations = PacketSelectorMutator::new(ScheduledPacketMutator::with_max_stack_pow(
        tuple_list!(
            TokenStreamInsertRandomMutator::new(max_tokens),
            TokenReplaceRandomMutator::new(),
            TokenSplitMutator::new(max_tokens),
            TokenStreamInsertInterestingMutator::new(max_tokens),
            TokenReplaceInterestingMutator::new(),
            TokenValueCopyMutator::new(),
            TokenValueInsertRandomMutator::new(),
            TokenStreamCopyMutator::new(max_tokens),
            TokenStreamSwapMutator::new(),
            TokenStreamDeleteMutator::new(0),
            TokenRotateCharMutator::new(),
            TokenValueDeleteMutator::new(0),
            TokenInsertSpecialCharMutator::new(),
            TokenStreamDictInsertMutator::new(max_tokens),
            TokenReplaceDictMutator::new(),
            TokenStreamScannerMutator::new(max_tokens),
        ),
        16,
    ));

    let mutational = StdMutationalStage::new(tt_mutations);

    let scheduler = RandScheduler::new();

    let mut fuzzer = StdFuzzer::new(scheduler, feedback, objective);

    let mut harness = print_token_stream;
    let mut executor = InProcessExecutor::new(&mut harness, tuple_list!(time_observer), &mut fuzzer, &mut state, &mut mgr)?;

    let mut stages = tuple_list!(mutational);

    let input = DragonflyInput::new(
        vec![
            TokenStream::builder()
                .text("PORT")
                .whitespace(" ")
                .number("127")
                .text(",")
                .number("0")
                .text(",")
                .number("0")
                .text(",")
                .number("1")
                .text(",")
                .number("655")
                .text(",")
                .number("35")
                .whitespace("\r\n")
                .build(),
        ]
    );
    fuzzer.add_input(&mut state, &mut executor, &mut mgr, input)?;

    fuzzer.fuzz_loop(&mut stages, &mut executor, &mut state, &mut mgr)?;

    Ok(())
}
