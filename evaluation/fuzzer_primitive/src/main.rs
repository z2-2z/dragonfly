use core::time::Duration;
use libafl::prelude::{
    current_nanos, StdRand, ShMem,
    ShMemProvider, UnixShMemProvider,
    StdShMemProvider,
    tuple_list,
    AsMutSlice,
    HasLen,
    Cores, CoreId,
    Launcher,
    InMemoryCorpus,
    feedback_or,
    CrashFeedback,
    MaxMapFeedback,
    TimeFeedback,
    Fuzzer, StdFuzzer,
    Input,
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
    BytesInput,
    HasBytesVec,
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
use dragonfly::prelude::{
    DragonflyExecutorBuilder,
    HasPacketVector,
    SerializeIntoBuffer,
    PacketDeleteMutator,
    PacketDuplicateMutator,
    PacketReorderMutator,
    NopMutator,
    StateObserver,
    StateFeedback,
    HasStateGraph,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
enum FTPPacket {
    Ctrl(BytesInput),
    Data(BytesInput),
    Sep,
}

impl SerializeIntoBuffer for FTPPacket {
    fn serialize_into_buffer(&self, buffer: &mut [u8]) -> Option<usize> {
        match self {
            FTPPacket::Data(data) |
            FTPPacket::Ctrl(data) => {
                let len = data.len();
                buffer[0..len].copy_from_slice(data.bytes());
                Some(len)
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

#[derive(Debug, Clone, Serialize, Deserialize)]
struct FTPInput {
    packets: Vec<FTPPacket>,
}

impl HasPacketVector for FTPInput {
    type Packet = FTPPacket;

    fn packets(&self) -> &[FTPPacket] {
        &self.packets
    }

    fn packets_mut(&mut self) -> &mut Vec<FTPPacket> {
        &mut self.packets
    }
}

impl Input for FTPInput {
    fn generate_name(&self, idx: usize) -> String {
        format!("{}", idx)
    }
}

impl HasLen for FTPInput {
    fn len(&self) -> usize {
        self.packets.len()
    }
}

fn main() {
    // For fuzzbench, crashes and finds are inside the same `corpus` directory, in the "queue" and "crashes" subdir.
    let out_dir = PathBuf::from("output");
    let _ = fs::create_dir(&out_dir);
    if !out_dir.is_dir() {
        println!("Out dir at {:?} is not a valid directory!", &out_dir);
        return;
    }

    let mut crashes = out_dir;
    crashes.push("crashes");

    let logfile = PathBuf::from("output/log");

    let timeout = Duration::from_millis(5000);

    let executable = "../proftpd/proftpd".to_string();

    let debug_child = true;

    let signal = str::parse::<Signal>("SIGKILL").unwrap();

    let arguments = vec![
        "-d".to_string(),
        "0".to_string(),
        "-q".to_string(),
        "-X".to_string(),
        "-c".to_string(),
        fs::canonicalize("../fuzz.conf").unwrap().to_str().unwrap().to_string(),
        "-n".to_string(),
    ];

    fuzz(crashes, &logfile, timeout, &executable, debug_child, signal, &arguments).expect("An error occurred while fuzzing");
}

/// The actual fuzzer
#[allow(clippy::too_many_arguments)]
fn fuzz(objective_dir: PathBuf, logfile: &PathBuf, timeout: Duration, executable: &str, debug_child: bool, signal: Signal, arguments: &[String]) -> Result<(), Error> {
    let mut client = |old_state: Option<_>, mut mgr, core: CoreId| {
        println!("Launch client @ {}", core.0);
        let mut shmem_provider = UnixShMemProvider::new()?;
        
        // The coverage map shared between observer and executor
        const MAP_SIZE: usize = 65536;
        let mut shmem = shmem_provider.new_shmem(MAP_SIZE)?;
        // let the forkserver know the shmid
        shmem.write_to_env("__AFL_SHM_ID")?;
        let shmem_buf = shmem.as_mut_slice();
        // To let know the AFL++ binary that we have a big map
        std::env::set_var("AFL_MAP_SIZE", format!("{}", MAP_SIZE));

        let state_observer = StateObserver::new(&mut shmem_provider, "StateObserver")?;

        // Create an observation channel using the hitcounts map of AFL++
        let edges_observer = HitcountsMapObserver::new(unsafe { StdMapObserver::new("shared_mem", shmem_buf) });

        // Create an observation channel to keep track of the execution time
        let time_observer = TimeObserver::new("time");
        
        let state_feedback = StateFeedback::new(&state_observer);

        let map_feedback = MaxMapFeedback::tracking(&edges_observer, true, false);

        let calibration = CalibrationStage::new(&map_feedback);

        // Feedback to rate the interestingness of an input
        // This one is composed by two Feedbacks in OR
        let mut feedback = feedback_or!(
            state_feedback,
            // New maximization map feedback linked to the edges observer and the feedback state
            map_feedback,
            // Time feedback, this one does not need a feedback state
            TimeFeedback::with_observer(&time_observer)
        );

        // A feedback to choose if an input is a solution or not
        let mut objective = CrashFeedback::new();

        // create a State from scratch
        let mut state = old_state.unwrap_or_else(|| StdState::new(
            // RNG
            StdRand::with_seed(current_nanos()),
            // Corpus that will be evolved, we keep it in memory for performance
            InMemoryCorpus::<FTPInput>::new(),
            // Corpus in which we store solutions (crashes in this example),
            // on disk so the user can get them after stopping the fuzzer
            InMemoryCorpus::new(),
            // States of the feedbacks.
            // The feedbacks can report the data that should persist in the State.
            &mut feedback,
            // Same for objective feedbacks
            &mut objective,
        ).unwrap());
        state.init_stategraph();

        let mutator = StdScheduledMutator::new(
            tuple_list!(
                /*
                PacketDeleteMutator::new(1),
                PacketDuplicateMutator::new(16),
                PacketReorderMutator::new()*/
                NopMutator::new(),
            )
        );

        let mutational = StdMutationalStage::new(mutator);

        let scheduler = RandScheduler::new();

        let mut fuzzer = StdFuzzer::new(scheduler, feedback, objective);

        let mut executor = DragonflyExecutorBuilder::new()
            .observers(tuple_list!(state_observer, edges_observer, time_observer))
            .shmem_provider(&mut shmem_provider)
            .timeout(timeout)
            .signal(signal)
            .debug_child(debug_child)
            .program(executable)
            .args(arguments)
            .is_deferred_forkserver(true)
            .build()?;

        // The order of the stages matter!
        let mut stages = tuple_list!(
            calibration, 
            mutational
        );

        // evaluate input
        let input = FTPInput {
            packets: vec![
                FTPPacket::Ctrl(BytesInput::new(b"USER ftp\r\n".to_vec())),
                FTPPacket::Sep,
                FTPPacket::Ctrl(BytesInput::new(b"PASS x\r\n".to_vec())),
                FTPPacket::Sep,
                FTPPacket::Ctrl(BytesInput::new(b"CWD uploads\r\n".to_vec())),
                FTPPacket::Sep,
                FTPPacket::Ctrl(BytesInput::new(b"EPSV\r\n".to_vec())),
                FTPPacket::Sep,
                FTPPacket::Ctrl(BytesInput::new(b"STOR fuzzertest.txt\r\n".to_vec())),
                FTPPacket::Data(BytesInput::new(b"it werks!!!".to_vec())),
                FTPPacket::Data(BytesInput::new(b"".to_vec())),
                FTPPacket::Sep,
                FTPPacket::Ctrl(BytesInput::new(b"QUIT\r\n".to_vec())),
            ],
        };
        fuzzer.evaluate_input(&mut state, &mut executor, &mut mgr, input)?;

        fuzzer.fuzz_loop(&mut stages, &mut executor, &mut state, &mut mgr)?;
        Ok(())
    };

    let cores = Cores::from_cmdline("0")?;
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
