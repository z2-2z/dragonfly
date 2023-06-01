use libafl::prelude::{
    current_nanos, StdRand,
    ShMemProvider,
    StdShMemProvider,
    tuple_list,
    Cores, CoreId,
    Launcher,
    InMemoryCorpus,
    OnDiskCorpus,
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
    EventConfig,
    ExitKind,
    InProcessExecutor,
    Tokens,
    HasMetadata,
};
use std::{
    fs,
    path::PathBuf,
};
use crate::{
    prelude::*,
    tt::*,
};

fn test_token_stream(input: &DragonflyInput<TokenStream>) -> ExitKind {
    let prev_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let result = std::panic::catch_unwind(|| {
        for packet in input.packets() {
            let mut builder = TokenStream::builder();
            
            for token in packet.tokens() {
                match token {
                    TextToken::Constant(data) => builder = builder.constant(data),
                    TextToken::Number(data) => builder = builder.number(std::str::from_utf8(data).unwrap()),
                    TextToken::Whitespace(data) => builder = builder.whitespace(std::str::from_utf8(data).unwrap()),
                    TextToken::Text(data) => builder = builder.text(std::str::from_utf8(data).unwrap()),
                    TextToken::Blob(data) => builder = builder.blob(data),
                }
            }
        }
    });
    std::panic::set_hook(prev_hook);
    if result.is_ok() {
        ExitKind::Ok
    } else {
        ExitKind::Crash
    }
}

#[test]
fn main() -> Result<(), Error> {
    let cores = "0";
    
    let out_dir = PathBuf::from("src/tests/tt_mutators/output");
    let _ = fs::create_dir(&out_dir);
    
    let mut crashes = out_dir;
    crashes.push("crashes");
    
    let seed = current_nanos();
    
    let mut client = |old_state: Option<_>, mut mgr, core: CoreId| {
        println!("Launch client @ {}", core.0);
        
        let time_observer = TimeObserver::new("time");

        let mut feedback = TimeFeedback::with_observer(&time_observer);

        let mut objective = CrashFeedback::new();

        let mut state = old_state.unwrap_or_else(|| StdState::new(
            StdRand::with_seed(seed),
            InMemoryCorpus::<DragonflyInput<TokenStream>>::new(),
            OnDiskCorpus::new(&crashes).expect("crashes"),
            &mut feedback,
            &mut objective,
        ).unwrap());
        state.init_stategraph();
        
        let mut tokens = Tokens::new();
        tokens.add_token(&b"TOKEN1-A".to_vec());
        tokens.add_token(&b"TOKEN2-B".to_vec());
        tokens.add_token(&b"TOKEN3-C".to_vec());
        tokens.add_token(&b"TOKEN4-D".to_vec());
        tokens.add_token(&b"TOKEN5-E".to_vec());
        state.add_metadata(tokens);
        
        let max_tokens = 128;
        let tt_mutations = ScheduledPacketMutator::new(
            tuple_list!(
                NopPacketMutator::new(),
                TokenStreamInsertRandomMutator::new(max_tokens),
                TokenReplaceRandomMutator::new(),
                TokenSplitMutator::new(max_tokens),
                TokenStreamInsertInterestingMutator::new(max_tokens),
                TokenReplaceInterestingMutator::new(),
                TokenStreamDuplicateMutator::new(max_tokens),
                TokenValueDuplicateMutator::new(),
                TokenValueInsertRandomMutator::new(),
                TokenStreamCopyMutator::new(max_tokens),
                TokenStreamSwapMutator::new(),
                TokenStreamDeleteMutator::new(0),
                TokenRepeatCharMutator::new(),
                TokenRotateCharMutator::new(),
                TokenValueDeleteMutator::new(0),
                TokenInsertSpecialCharMutator::new(),
                TokenInvertCaseMutator::new(),
                TokenStreamDictInsertMutator::new(max_tokens),
                TokenReplaceDictMutator::new(),
                TokenStreamScannerMutator::new(max_tokens),
                TokenTransformConstantMutator::new()
            )
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
                TokenStream::builder().constant("").whitespace("").text("").constant("").build()
            ]
        );
        fuzzer.add_input(&mut state, &mut executor, &mut mgr, input)?;

        fuzzer.fuzz_loop(&mut stages, &mut executor, &mut state, &mut mgr)?;
        
        println!("Stopping client {}", core.0);
        Ok(())
    };

    let cores = Cores::from_cmdline(cores)?;
    let monitor = SimplePrintingMonitor::new();

    let mut launcher = Launcher::builder()
        .shmem_provider(StdShMemProvider::new()?)
        .configuration(EventConfig::from_name("default"))
        .run_client(&mut client)
        .cores(&cores)
        .monitor(monitor)
        .broker_port(1337)
        .remote_broker_addr(Some("127.0.0.1:1337".parse().unwrap()))
        .build();

    match launcher.launch() {
        Ok(_) => {},
        Err(Error::ShuttingDown) => {},
        err => panic!("Failed to lauch instances: {:?}", err)
    };

    Ok(())
}
