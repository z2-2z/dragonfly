use core::time::Duration;
use libafl::prelude::{
    current_nanos, StdRand, ShMem,
    ShMemProvider, UnixShMemProvider,
    StdShMemProvider,
    tuple_list,
    AsMutSlice,
    Cores, CoreId,
    Launcher,
    CachedOnDiskCorpus,
    OnDiskCorpus,
    feedback_or,
    CrashFeedback,
    MaxMapFeedback,
    TimeFeedback,
    Fuzzer, StdFuzzer,
    OnDiskTOMLMonitor,
    SimplePrintingMonitor,
    StdScheduledMutator,
    HitcountsMapObserver,
    StdMapObserver,
    TimeObserver,
    RandScheduler,
    CalibrationStage,
    StdMutationalStage,
    StdState,
    Error,
    Evaluator,
    EventConfig,
};
use nix::sys::signal::Signal;
use serde::{
    Deserialize,
    Serialize,
};
use std::{
    fs,
    path::PathBuf,
};
use dragonfly::{
    prelude::{
        LibdragonflyExecutorBuilder,
        SerializeIntoBuffer,
        PacketDeleteMutator,
        PacketDuplicateMutator,
        PacketReorderMutator,
        ScheduledPacketMutator,
        StateObserver,
        StateFeedback,
        HasStateGraph,
        DragonflyInput,
    },
    tt::{
        TokenStream,
        HasTokenStream,
        RandomTokenValueMutator
    },
};

#[derive(Debug, Clone, Serialize, Deserialize, Hash)]
enum FTPPacket {
    Ctrl(TokenStream),
    Data(TokenStream),
    Sep,
}

impl SerializeIntoBuffer for FTPPacket {
    fn serialize_into_buffer(&self, buffer: &mut [u8]) -> Option<usize> {
        match self {
            FTPPacket::Data(data) |
            FTPPacket::Ctrl(data) => {
                Some(data.generate_text(buffer))
            },
            FTPPacket::Sep => None,
        }
    }

    fn get_connection(&self) -> usize {
        match self {
            FTPPacket::Ctrl(_) => 0,
            FTPPacket::Data(_) => 1,
            FTPPacket::Sep => unreachable!(),
        }
    }

    fn terminates_group(&self) -> bool {
        matches!(self, FTPPacket::Sep)
    }
}

impl HasTokenStream for FTPPacket {
    fn get_tokenstream(&mut self) -> Option<&mut TokenStream> {
        match self {
            FTPPacket::Ctrl(data) |
            FTPPacket::Data(data) => Some(data),
            FTPPacket::Sep => None,
        }
    }
}

fn main() -> Result<(), Error> {
    let args: Vec<String> = std::env::args().collect();
    
    let cores = args.get(1).map(|x| x.as_str()).unwrap_or("0");
    
    let out_dir = PathBuf::from("output");
    let _ = fs::create_dir(&out_dir);
    
    let mut crashes = out_dir.clone();
    crashes.push("crashes");
    
    let mut queue = out_dir.clone();
    queue.push("queue");
    
    let mut logfile = out_dir;
    logfile.push("log");
    
    let timeout = Duration::from_millis(5000);
    
    let executable = "../proftpd/proftpd".to_string();
    let arguments = vec![
        "-d".to_string(),
        "0".to_string(),
        "-q".to_string(),
        "-X".to_string(),
        "-c".to_string(),
        fs::canonicalize("../fuzz.conf").unwrap().to_str().unwrap().to_string(),
        "-n".to_string(),
    ];
    
    let debug_child = true;
    
    let signal = str::parse::<Signal>("SIGKILL").unwrap();
    
    let seed = current_nanos();
    
    let mut client = |old_state: Option<_>, mut mgr, core: CoreId| {
        println!("Launch client @ {}", core.0);
        
        let mut shmem_provider = UnixShMemProvider::new()?;
        const MAP_SIZE: usize = 65536;
        let mut shmem = shmem_provider.new_shmem(MAP_SIZE)?;
        shmem.write_to_env("__AFL_SHM_ID")?;
        let shmem_buf = shmem.as_mut_slice();
        std::env::set_var("AFL_MAP_SIZE", format!("{}", MAP_SIZE));

        let state_observer = StateObserver::new(&mut shmem_provider, "StateObserver")?;
        let edges_observer = HitcountsMapObserver::new(unsafe { StdMapObserver::new("shared_mem", shmem_buf) });
        let time_observer = TimeObserver::new("time");
        
        let state_feedback = StateFeedback::new(&state_observer);
        let map_feedback = MaxMapFeedback::tracking(&edges_observer, true, false);

        let calibration = CalibrationStage::new(&map_feedback);

        let mut feedback = feedback_or!(
            state_feedback,
            map_feedback,
            TimeFeedback::with_observer(&time_observer)
        );

        let mut objective = CrashFeedback::new();

        let mut state = old_state.unwrap_or_else(|| StdState::new(
            StdRand::with_seed(seed),
            CachedOnDiskCorpus::<DragonflyInput<FTPPacket>>::new(&queue, 128).expect("queue"),
            OnDiskCorpus::<DragonflyInput<FTPPacket>>::new(&crashes).expect("crashes"),
            &mut feedback,
            &mut objective,
        ).unwrap());
        state.init_stategraph();
        
        let packet_mutator = ScheduledPacketMutator::new(
            tuple_list!(
                RandomTokenValueMutator::new()
            )
        );

        let mutator = StdScheduledMutator::with_max_stack_pow(
            tuple_list!(
                PacketDeleteMutator::new(1),
                PacketDuplicateMutator::new(16),
                PacketReorderMutator::new(),
                packet_mutator
            ),
            0
        );

        let mutational = StdMutationalStage::new(mutator);

        let scheduler = RandScheduler::new();

        let mut fuzzer = StdFuzzer::new(scheduler, feedback, objective);

        let mut executor = LibdragonflyExecutorBuilder::new()
            .observers(tuple_list!(state_observer, edges_observer, time_observer))
            .shmem_provider(&mut shmem_provider)
            .timeout(timeout)
            .signal(signal)
            .debug_child(debug_child)
            .program(&executable)
            .args(&arguments)
            .is_deferred_forkserver(true)
            .build()?;

        let mut stages = tuple_list!(
            calibration, 
            mutational
        );

        let input = DragonflyInput::new(
            vec![
                FTPPacket::Ctrl(
                    TokenStream::builder().constant("USER").whitespace(" ").text("ftp").constant("\r\n").build()
                ),
                FTPPacket::Sep,
                FTPPacket::Ctrl(
                    TokenStream::builder().constant("PASS").whitespace(" ").text("x").constant("\r\n").build()
                ),
                FTPPacket::Sep,
                FTPPacket::Ctrl(
                    TokenStream::builder().constant("CWD").whitespace(" ").text("uploads").constant("\r\n").build()
                ),
                FTPPacket::Sep,
                FTPPacket::Ctrl(
                    TokenStream::builder().constant("EPSV").constant("\r\n").build()
                ),
                FTPPacket::Sep,
                FTPPacket::Ctrl(
                    TokenStream::builder().constant("STOR").whitespace(" ").text("fuzzertest.txt").constant("\r\n").build()
                ),
                FTPPacket::Data(
                    TokenStream::builder().blob(b"it werks!!!").build()
                ),
                FTPPacket::Data(
                    TokenStream::builder().build()
                ),
                FTPPacket::Sep,
                FTPPacket::Ctrl(
                    TokenStream::builder().constant("QUIT").constant("\r\n").build()
                ),
            ]
        );
        fuzzer.evaluate_input(&mut state, &mut executor, &mut mgr, input)?;

        fuzzer.fuzz_loop_for(&mut stages, &mut executor, &mut state, &mut mgr, 1)?;
        
        println!("Stopping client {}", core.0);
        Ok(())
    };

    let cores = Cores::from_cmdline(cores)?;
    let monitor = OnDiskTOMLMonitor::new(logfile, SimplePrintingMonitor::new());

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
