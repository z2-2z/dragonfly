use core::{
    cell::RefCell,
    time::Duration,
};
use libafl::{
    bolts::{
        current_nanos,
        current_time,
        rands::StdRand,
        shmem::{
            ShMem,
            ShMemProvider,
            UnixShMemProvider,
        },
        tuples::tuple_list,
        AsMutSlice,
        HasLen,
    },
    corpus::{
        InMemoryCorpus,
        OnDiskCorpus,
    },
    events::SimpleEventManager,
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
    inputs::{
        BytesInput,
        Input,
    },
    monitors::SimpleMonitor,
    mutators::StdMOptMutator,
    observers::{
        HitcountsMapObserver,
        StdMapObserver,
        TimeObserver,
    },
    schedulers::{
        powersched::PowerSchedule,
        IndexesLenTimeMinimizerScheduler,
        StdWeightedScheduler,
    },
    stages::{
        calibrate::CalibrationStage,
        power::StdPowerMutationalStage,
    },
    state::StdState,
    Error,
    Evaluator,
};
use nix::sys::signal::Signal;
use serde::{
    Deserialize,
    Serialize,
};
use std::{
    fs::{
        self,
        OpenOptions,
    },
    io::Write,
    path::PathBuf,
};

use crate::{
    executor::DragonflyExecutorBuilder,
    input::HasPacketVector,
    mutators::NopMutator,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ExampleInput {
    packets: Vec<BytesInput>,
}

impl HasPacketVector for ExampleInput {
    type Packet = BytesInput;

    fn packets(&self) -> &[BytesInput] {
        &self.packets
    }

    fn packets_mut(&mut self) -> &mut Vec<BytesInput> {
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
        // needed for LenTimeMulTestcaseScore so lets return the sum of all packet lengths
        let mut sum = 0;

        for packet in self.packets() {
            sum += packet.len();
        }

        sum
    }
}

#[test]
fn packet_channel() {
    affinity::set_thread_affinity([0]).unwrap();

    println!("Workdir: {:?}", std::env::current_dir().unwrap().to_string_lossy().to_string());

    // For fuzzbench, crashes and finds are inside the same `corpus` directory, in the "queue" and "crashes" subdir.
    let out_dir = PathBuf::from("src/tests/packet_channel/output");
    let _ = fs::create_dir(&out_dir);
    if !out_dir.is_dir() {
        println!("Out dir at {:?} is not a valid directory!", &out_dir);
        return;
    }

    let mut crashes = out_dir;
    crashes.push("crashes");

    let logfile = PathBuf::from("src/tests/packet_channel/output/log");

    let timeout = Duration::from_millis(5000);

    let executable = "src/tests/packet_channel/test";

    let debug_child = true;

    let signal = str::parse::<Signal>("SIGKILL").unwrap();

    let arguments = Vec::new();

    fuzz(crashes, &logfile, timeout, executable, debug_child, signal, &arguments).expect("An error occurred while fuzzing");
}

/// The actual fuzzer
#[allow(clippy::too_many_arguments)]
fn fuzz(objective_dir: PathBuf, logfile: &PathBuf, timeout: Duration, executable: &str, debug_child: bool, signal: Signal, arguments: &[String]) -> Result<(), Error> {
    // a large initial map size that should be enough
    // to house all potential coverage maps for our targets
    // (we will eventually reduce the used size according to the actual map)
    const MAP_SIZE: usize = 65536;

    let log = RefCell::new(OpenOptions::new().append(true).create(true).open(logfile)?);

    // 'While the monitor are state, they are usually used in the broker - which is likely never restarted
    let monitor = SimpleMonitor::new(|s| {
        println!("{s}");
        writeln!(log.borrow_mut(), "{:?} {}", current_time(), s).unwrap();
    });

    // The event manager handle the various events generated during the fuzzing loop
    // such as the notification of the addition of a new item to the corpus
    let mut mgr = SimpleEventManager::new(monitor);

    // The unix shmem provider for shared memory, to match AFL++'s shared memory at the target side
    let mut shmem_provider = UnixShMemProvider::new().unwrap();

    // The coverage map shared between observer and executor
    let mut shmem = shmem_provider.new_shmem(MAP_SIZE).unwrap();
    // let the forkserver know the shmid
    shmem.write_to_env("__AFL_SHM_ID").unwrap();
    let shmem_buf = shmem.as_mut_slice();
    // To let know the AFL++ binary that we have a big map
    std::env::set_var("AFL_MAP_SIZE", format!("{}", MAP_SIZE));

    // Create an observation channel using the hitcounts map of AFL++
    let edges_observer = HitcountsMapObserver::new(unsafe { StdMapObserver::new("shared_mem", shmem_buf) });

    // Create an observation channel to keep track of the execution time
    let time_observer = TimeObserver::new("time");

    let map_feedback = MaxMapFeedback::tracking(&edges_observer, true, false);

    let calibration = CalibrationStage::new(&map_feedback);

    // Feedback to rate the interestingness of an input
    // This one is composed by two Feedbacks in OR
    let mut feedback = feedback_or!(
        // New maximization map feedback linked to the edges observer and the feedback state
        map_feedback,
        // Time feedback, this one does not need a feedback state
        TimeFeedback::with_observer(&time_observer)
    );

    // A feedback to choose if an input is a solution or not
    let mut objective = CrashFeedback::new();

    // create a State from scratch
    let mut state = StdState::new(
        // RNG
        StdRand::with_seed(current_nanos()),
        // Corpus that will be evolved, we keep it in memory for performance
        InMemoryCorpus::<ExampleInput>::new(),
        // Corpus in which we store solutions (crashes in this example),
        // on disk so the user can get them after stopping the fuzzer
        OnDiskCorpus::new(objective_dir).unwrap(),
        // States of the feedbacks.
        // The feedbacks can report the data that should persist in the State.
        &mut feedback,
        // Same for objective feedbacks
        &mut objective,
    )
    .unwrap();

    // Setup a MOPT mutator
    let mutator = StdMOptMutator::new(&mut state, tuple_list!(NopMutator::new()), 7, 5)?;

    let power = StdPowerMutationalStage::new(mutator);

    // A minimization+queue policy to get testcasess from the corpus
    let scheduler = IndexesLenTimeMinimizerScheduler::new(StdWeightedScheduler::with_schedule(&mut state, &edges_observer, Some(PowerSchedule::EXPLORE)));

    // A fuzzer with feedbacks and a corpus scheduler
    let mut fuzzer = StdFuzzer::new(scheduler, feedback, objective);

    let mut executor = DragonflyExecutorBuilder::new()
        .observers(tuple_list!(edges_observer, time_observer))
        .shmem_provider(&mut shmem_provider)
        .timeout(timeout)
        .signal(signal)
        .debug_child(debug_child)
        .program(executable)
        .args(arguments)
        .is_deferred_forkserver(true)
        .env("LD_LIBRARY_PATH", "src/tests/packet_channel/build")
        .build()?;

    // The order of the stages matter!
    let mut stages = tuple_list!(calibration, power);

    // evaluate input
    let input = ExampleInput {
        packets: vec![BytesInput::new(b"Hello".to_vec()), BytesInput::new(b"x".to_vec()), BytesInput::new(b"World".to_vec())],
    };
    fuzzer.evaluate_input(&mut state, &mut executor, &mut mgr, input)?;

    //fuzzer.fuzz_loop_for(&mut stages, &mut executor, &mut state, &mut mgr, 1)?;
    fuzzer.fuzz_loop(&mut stages, &mut executor, &mut state, &mut mgr)?;

    Ok(())
}
