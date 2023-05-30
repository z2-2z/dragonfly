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
    CachedOnDiskCorpus,
    OnDiskCorpus,
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
            CachedOnDiskCorpus::<FTPInput>::new(&queue, 128).expect("queue"),
            OnDiskCorpus::<FTPInput>::new(&crashes).expect("crashes"),
            &mut feedback,
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
            .program(&executable)
            .args(&arguments)
            .is_deferred_forkserver(true)
            .build()?;

        let mut stages = tuple_list!(
            calibration, 
            mutational
        );

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