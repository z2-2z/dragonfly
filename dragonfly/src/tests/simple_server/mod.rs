use core::time::Duration;
use libafl::{
    bolts::{
        current_nanos,
        rands::StdRand,
        shmem::{
            ShMem,
            ShMemProvider,
            UnixShMemProvider,
            StdShMemProvider,
        },
        tuples::tuple_list,
        AsMutSlice,
        HasLen,
        core_affinity::{Cores, CoreId},
        launcher::Launcher,
    },
    corpus::{
        InMemoryCorpus,
    },
    feedback_or,
    feedbacks::{
        CrashFeedback,
        MaxMapFeedback,
        TimeFeedback,
    },
    fuzzer::{
        Fuzzer,
        StdFuzzer,
    },
    inputs::Input,
    monitors::{
        OnDiskTOMLMonitor,
        SimplePrintingMonitor,
    },
    mutators::StdScheduledMutator,
    observers::{
        HitcountsMapObserver,
        StdMapObserver,
        TimeObserver,
    },
    schedulers::RandScheduler,
    stages::{
        calibrate::CalibrationStage,
        mutational::StdMutationalStage,
    },
    state::StdState,
    Error,
    Evaluator,
    events::EventConfig,
};
use nix::sys::signal::Signal;
use serde::{
    Deserialize,
    Serialize,
};
use std::{
    fs::{self,},
    path::PathBuf,
};

use crate::{
    executor::LibdragonflyExecutorBuilder,
    input::{
        HasPacketVector,
        SerializeIntoBuffer,
    },
    mutators::{
        PacketDeleteMutator,
        PacketDuplicateMutator,
        PacketReorderMutator,
    },
    observer::StateObserver,
    feedback::StateFeedback,
    graph::HasStateGraph,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
enum Packet {
    Add4,
    Sub1,
    NegS,
    Sep,
}

impl SerializeIntoBuffer for Packet {
    fn serialize_into_buffer(&self, buffer: &mut [u8]) -> Option<usize> {
        match self {
            Packet::Add4 => {
                buffer[0..4].copy_from_slice(b"add4");
                Some(4)
            },
            Packet::Sub1 => {
                buffer[0..4].copy_from_slice(b"sub1");
                Some(4)
            },
            Packet::NegS => {
                buffer[0..4].copy_from_slice(b"negs");
                Some(4)
            },
            Packet::Sep => None,
        }
    }

    fn get_connection(&self) -> usize {
        0
    }

    fn terminates_group(&self) -> bool {
        matches!(self, Packet::Sep)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ExampleInput {
    packets: Vec<Packet>,
}

impl HasPacketVector for ExampleInput {
    type Packet = Packet;

    fn packets(&self) -> &[Packet] {
        &self.packets
    }

    fn packets_mut(&mut self) -> &mut Vec<Packet> {
        &mut self.packets
    }
}

impl Input for ExampleInput {
    fn generate_name(&self, idx: usize) -> String {
        format!("{}", idx)
    }
}

impl HasLen for ExampleInput {
    fn len(&self) -> usize {
        self.packets.len()
    }
}

#[test]
fn simple_server() {
    let args: Vec<String> = std::env::args().collect();
    println!("args: {:?}", args);
    println!("Workdir: {:?}", std::env::current_dir().unwrap().to_string_lossy().to_string());

    // For fuzzbench, crashes and finds are inside the same `corpus` directory, in the "queue" and "crashes" subdir.
    let out_dir = PathBuf::from("src/tests/simple_server/output");
    let _ = fs::create_dir(&out_dir);
    if !out_dir.is_dir() {
        println!("Out dir at {:?} is not a valid directory!", &out_dir);
        return;
    }

    let mut crashes = out_dir;
    crashes.push("crashes");

    let logfile = PathBuf::from("src/tests/simple_server/output/log");

    let timeout = Duration::from_millis(5000);

    let executable = format!("src/tests/simple_server/{}", args[args.len() - 1]);

    let debug_child = true;

    let signal = str::parse::<Signal>("SIGKILL").unwrap();

    let arguments = Vec::new();

    fuzz(crashes, &logfile, timeout, &executable, debug_child, signal, &arguments).expect("An error occurred while fuzzing");
}

/// The actual fuzzer
#[allow(clippy::too_many_arguments)]
fn fuzz(_objective_dir: PathBuf, logfile: &PathBuf, timeout: Duration, executable: &str, debug_child: bool, signal: Signal, arguments: &[String]) -> Result<(), Error> {
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
            InMemoryCorpus::<ExampleInput>::new(),
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

        let mutator = StdScheduledMutator::new(tuple_list!(PacketDeleteMutator::new(1), PacketDuplicateMutator::new(16), PacketReorderMutator::new()));

        let mutational = StdMutationalStage::new(mutator);

        let scheduler = RandScheduler::new();

        let mut fuzzer = StdFuzzer::new(scheduler, feedback, objective);

        let mut executor = LibdragonflyExecutorBuilder::new()
            .observers(tuple_list!(state_observer, edges_observer, time_observer))
            .shmem_provider(&mut shmem_provider)
            .timeout(timeout)
            .signal(signal)
            .debug_child(debug_child)
            .program(executable)
            .args(arguments)
            .is_deferred_forkserver(true)
            .env("LD_LIBRARY_PATH", "src/tests/simple_server/build")
            .build()?;

        // The order of the stages matter!
        let mut stages = tuple_list!(
            calibration, 
            mutational
        );

        // evaluate input
        let input = ExampleInput {
            packets: vec![Packet::Add4, Packet::Sep, Packet::Sub1, Packet::Sep, Packet::NegS, Packet::Sep],
        };
        fuzzer.add_input(&mut state, &mut executor, &mut mgr, input)?;
        
        println!("Start fuzzing @ {}", core.0);

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
