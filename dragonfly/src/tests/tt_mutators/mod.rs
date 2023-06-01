use libafl::prelude::{
    current_nanos, StdRand,
    tuple_list,
    InMemoryCorpus,
    CrashFeedback,
    TimeFeedback,
    Fuzzer, StdFuzzer,
    SimplePrintingMonitor,
    TimeObserver,
    RandScheduler,
    StdMutationalStage,
    StdState,
    Error,
    Evaluator,
    ExitKind,
    InProcessExecutor,
    Tokens,
    HasMetadata,
    SimpleEventManager,
};
use crate::{
    prelude::*,
    tt::*,
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
fn main() -> Result<(), Error> {
    let seed = current_nanos();
    
    let monitor = SimplePrintingMonitor::new();
    let mut mgr = SimpleEventManager::new(monitor);
        
    let time_observer = TimeObserver::new("time");

    let mut feedback = TimeFeedback::with_observer(&time_observer);

    let mut objective = CrashFeedback::new();

    let mut state = StdState::new(
        StdRand::with_seed(seed),
        InMemoryCorpus::<DragonflyInput<TokenStream>>::new(),
        InMemoryCorpus::new(),
        &mut feedback,
        &mut objective,
    ).unwrap();
    state.init_stategraph();
    
    let mut tokens = Tokens::new();
    tokens.add_token(&b"TOKEN1-A".to_vec());
    tokens.add_token(&b"TOKEN2-B".to_vec());
    tokens.add_token(&b"TOKEN3-C".to_vec());
    tokens.add_token(&b"TOKEN4-D".to_vec());
    tokens.add_token(&b"TOKEN5-E".to_vec());
    state.add_metadata(tokens);
    
    let max_tokens = 16;
    let tt_mutations = ScheduledPacketMutator::with_max_stack_pow(
        tuple_list!(
            /*  0 */ NopPacketMutator::new(),
            /*  1 */ TokenStreamInsertRandomMutator::new(max_tokens),
            /*  2 */ TokenReplaceRandomMutator::new(),
            /*  3 */ TokenSplitMutator::new(max_tokens),
            /*  4 */ TokenStreamInsertInterestingMutator::new(max_tokens),
            /*  5 */ TokenReplaceInterestingMutator::new(),
            /*  6 */ TokenStreamDuplicateMutator::new(max_tokens),
            /*  7 */ TokenValueDuplicateMutator::new(),
            /*  8 */ TokenValueInsertRandomMutator::new(),
            /*  9 */ TokenStreamCopyMutator::new(max_tokens),
            /* 10 */ TokenStreamSwapMutator::new(),
            /* 11 */ TokenStreamDeleteMutator::new(0),
            /* 12 */ TokenRepeatCharMutator::new(),
            /* 13 */ TokenRotateCharMutator::new(),
            /* 14 */ TokenValueDeleteMutator::new(0),
            /* 15 */ TokenInsertSpecialCharMutator::new(),
            /* 16 */ TokenInvertCaseMutator::new(),
            /* 17 */ TokenStreamDictInsertMutator::new(max_tokens),
            /* 18 */ TokenReplaceDictMutator::new(),
            /* 19 */ TokenStreamScannerMutator::new(max_tokens),
            /* 20 */ TokenTransformConstantMutator::new()
        ),
        16
    );

    let mutational = StdMutationalStage::new(tt_mutations);

    let scheduler = RandScheduler::new();

    let mut fuzzer = StdFuzzer::new(scheduler, feedback, objective);

    let mut harness = test_token_stream;
    let mut executor = InProcessExecutor::new(
        &mut harness,
        tuple_list!(time_observer),
        &mut fuzzer,
        &mut state,
        &mut mgr,
    )?;

    let mut stages = tuple_list!(
        mutational
    );

    let input = DragonflyInput::new(
        vec![
            TokenStream::builder().constant("").whitespace("").text("").blob("").build(),
            TokenStream::builder().build(),
        ]
    );
    fuzzer.add_input(&mut state, &mut executor, &mut mgr, input)?;

    fuzzer.fuzz_loop(&mut stages, &mut executor, &mut state, &mut mgr)?;

    Ok(())
}
