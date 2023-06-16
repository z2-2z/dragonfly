use core::time::Duration;
use libafl::prelude::{
    current_nanos, StdRand, ShMem,
    ShMemProvider, UnixShMemProvider,
    tuple_list,
    AsMutSlice,
    CachedOnDiskCorpus,
    OnDiskCorpus,
    feedback_or,
    CrashFeedback,
    MaxMapFeedback,
    TimeFeedback,
    Fuzzer, StdFuzzer,
    OnDiskJSONMonitor,
    StdScheduledMutator,
    HitcountsMapObserver,
    StdMapObserver,
    TimeObserver,
    IndexesLenTimeMinimizerScheduler,
    CalibrationStage,
    StdPowerMutationalStage,
    StdState,
    Error,
    Evaluator,
    SimpleEventManager,
    HasRand,
    Rand,
    HasMetadata,
    Tokens,
    current_time,
    Input,
    CoreId,
    HasSolutions,
    Corpus,
    powersched::PowerSchedule,
    StdWeightedScheduler,
    RandScheduler,
    StdMutationalStage,
};
use nix::sys::signal::Signal;
use serde::{
    Deserialize,
    Serialize,
};
use std::{
    fs,
    path::PathBuf,
    io::Write,
    fs::File,
};
use dragonfly::{
    prelude::{
        DragonflyExecutorBuilder,
        SerializeIntoBuffer,
        PacketDeleteMutator,
        PacketDuplicateMutator,
        PacketReorderMutator,
        ScheduledPacketMutator,
        StateObserver,
        StateFeedback,
        HasStateGraph,
        DragonflyInput,
        InsertRandomPacketMutator, NewRandom,
        HasPacketVector,
        InsertGeneratedPacketMutator, NewGenerated,
        HasCrossover, PacketCrossoverInsertMutator, PacketCrossoverReplaceMutator,
        StateAwareWeightedScheduler,
        PacketRepeatMutator,
    },
    tt::{
        TokenStream,
        HasTokenStream,
        TokenStreamInsertRandomMutator,
        TokenReplaceRandomMutator,
        TokenSplitMutator,
        TokenReplaceInterestingMutator,
        TokenStreamInsertInterestingMutator,
        TokenStreamDuplicateMutator,
        TokenValueDuplicateMutator,
        TokenValueInsertRandomMutator,
        TokenStreamCopyMutator,
        TokenStreamSwapMutator,
        TokenStreamDeleteMutator,
        TokenRepeatCharMutator,
        TokenRotateCharMutator,
        TokenValueDeleteMutator,
        TokenInsertSpecialCharMutator,
        TokenInvertCaseMutator,
        TokenStreamDictInsertMutator,
        TokenReplaceDictMutator,
        TokenStreamScannerMutator,
        TokenConvertMutator,
        TokenReplaceSpecialCharMutator,
    },
};
use clap::{Parser, CommandFactory};
use std::fmt::Display;
use std::path::Path;
#[cfg(debug_assertions)]
use libafl::prelude::SimplePrintingMonitor;
#[cfg(not(debug_assertions))]
use libafl::prelude::NopMonitor;

#[link(name = "generator")]
extern "C" {
    fn ftp_generator_seed(initial_seed: usize);
    fn ftp_generator_generate(buf: *mut u8, len: usize) -> usize;
}

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

impl<S> NewRandom<S> for FTPPacket 
where
    S: HasRand + HasMetadata,
{
    fn new_random(state: &mut S) -> Self {
        let typ = state.rand_mut().below(3);
        
        match typ {
            0 => FTPPacket::Ctrl(TokenStream::new_random(state)),
            1 => FTPPacket::Data(TokenStream::new_random(state)),
            2 => FTPPacket::Sep,
            _ => unreachable!()
        }
    }
}

impl Display for FTPPacket {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            FTPPacket::Ctrl(data) => write!(f, "Ctrl({})", data),
            FTPPacket::Data(data) => write!(f, "Data({})", data),
            FTPPacket::Sep => write!(f, "Sep"),
        }
    }
}

impl<S> NewGenerated<S> for FTPPacket 
where
    S: HasRand,
{
    fn new_generated(state: &mut S) -> Self {
        let seed = state.rand_mut().next() as usize;
        
        assert!(seed != 0);
        
        unsafe {
            ftp_generator_seed(seed);
        }
        
        let mut buf = [0; 32];
        
        let len = unsafe {
            ftp_generator_generate(buf.as_mut_ptr(), buf.len())
        };
        
        assert!(len <= buf.len());
        
        let stream = TokenStream::builder().text(&buf[..len]).build();
        FTPPacket::Ctrl(stream)
    }
}

impl<S> HasCrossover<S> for FTPPacket 
where
    S: HasRand,
{
    fn crossover_insert(&mut self, state: &mut S, other: Self) {
        match self {
            FTPPacket::Ctrl(data) |
            FTPPacket::Data(data) => {
                match other {
                    FTPPacket::Ctrl(other_data) |
                    FTPPacket::Data(other_data) => {
                        data.crossover_insert(state, other_data);
                    },
                    FTPPacket::Sep => {},
                }
            },
            FTPPacket::Sep => {},
        }
    }

    fn crossover_replace(&mut self, state: &mut S, other: Self) {
        match self {
            FTPPacket::Ctrl(data) |
            FTPPacket::Data(data) => {
                match other {
                    FTPPacket::Ctrl(other_data) |
                    FTPPacket::Data(other_data) => {
                        data.crossover_replace(state, other_data);
                    },
                    FTPPacket::Sep => {},
                }
            },
            FTPPacket::Sep => {},
        }
    }
}

#[derive(clap::Parser)]
struct Args {
    #[arg(short, long)]
    output: Option<String>,
    
    #[arg(short, long)]
    print: Option<String>,
    
    #[arg(short, long)]
    replay: Option<String>,
    
    #[arg(short, long)]
    debug: bool,
    
    #[arg(short, long)]
    trace: bool,
    
    #[arg(short, long)]
    ipsm: Option<String>,
}

fn main() -> Result<(), Error> {
    let args = Args::parse();
    
    if let Some(input_file) = &args.print {
        let input = DragonflyInput::<FTPPacket>::from_file(input_file)?;
        
        for packet in input.packets() {
            println!("{}", packet);
        }
        
        std::process::exit(0);
    }
    
    CoreId(0).set_affinity()?;
    
    let output = args.output.unwrap_or_else(|| {
        Args::command()
            .error(
                clap::error::ErrorKind::ArgumentConflict,
                "Output folder not specified"
            )
            .exit();
    });
    let out_dir = PathBuf::from(output);
    let _ = fs::create_dir(&out_dir);
    
    let mut crashes = out_dir.clone();
    crashes.push("crashes");
    
    let mut queue = out_dir.clone();
    queue.push("queue");
    
    let mut logfile = out_dir;
    logfile.push("log");
    
    #[cfg(debug_assertions)]
    let timeout = Duration::from_secs(10000);
    #[cfg(not(debug_assertions))]
    let timeout = Duration::from_millis(5000);
    
    let mut executable = "../proftpd/proftpd".to_string();
    
    #[cfg(debug_assertions)]
    let debug_level = "10";
    #[cfg(not(debug_assertions))]
    let debug_level = "0";
    
    let mut arguments = vec![
        "-d".to_string(),
        debug_level.to_string(),
        "-q".to_string(),
        "-X".to_string(),
        "-c".to_string(),
        fs::canonicalize("./fuzz.conf").unwrap().to_str().unwrap().to_string(),
        "-n".to_string(),
    ];
    const SUDO_ARGUMENTS: [&str; 6] = [
        "-C", "256",
        "-E",
        "-n",
        "-P",
        "--",
    ];
    
    if args.debug {
        arguments.splice(0..0, [
            "gdbserver".to_string(),
            "0.0.0.0:6666".to_string(),
            executable,
        ]);
        arguments.splice(0..0, SUDO_ARGUMENTS.iter().map(|x| x.to_string()));
        executable = "sudo".to_string();
    } else if args.trace {
        arguments.splice(0..0, [
            "strace".to_string(),
            "-f".to_string(),
            "--signal=!SIGCHLD".to_string(),
            "--trace=all".to_string(),
            "--".to_string(),
            executable,
        ]);
        arguments.splice(0..0, SUDO_ARGUMENTS.iter().map(|x| x.to_string()));
        executable = "sudo".to_string();
    }
    
    #[cfg(debug_assertions)]
    let debug_child = true;
    #[cfg(not(debug_assertions))]
    let debug_child = false;
    
    let signal = str::parse::<Signal>("SIGKILL").unwrap();
    
    let seed = current_nanos();
    
    let mut last_updated = 0;
    
    #[cfg(debug_assertions)]
    let inner_monitor = SimplePrintingMonitor::new();
    #[cfg(not(debug_assertions))]
    let inner_monitor = NopMonitor::new();
    
    let monitor = OnDiskJSONMonitor::new(
        logfile,
        inner_monitor,
        move |_| {
            let now = current_time().as_secs();
            
            if (now - last_updated) >= 60 {
                last_updated = now;
                true
            } else {
                false
            }
        }
    );
    let mut mgr = SimpleEventManager::new(monitor);
        
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
    
    let dictionary = Tokens::from_file("../ftp.dict")?;
    
    let mut state = StdState::new(
        StdRand::with_seed(seed),
        CachedOnDiskCorpus::<DragonflyInput<FTPPacket>>::new(&queue, 128)?,
        OnDiskCorpus::<DragonflyInput<FTPPacket>>::new(&crashes)?,
        &mut feedback,
        &mut objective,
    )?;
    state.init_stategraph();
    state.add_metadata(dictionary);
    
    let max_tokens = 256;
    let packet_mutator = ScheduledPacketMutator::with_max_stack_pow(
        tuple_list!(
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
            TokenStreamDeleteMutator::new(1),
            TokenRepeatCharMutator::new(),
            TokenRotateCharMutator::new(),
            TokenValueDeleteMutator::new(1),
            TokenInsertSpecialCharMutator::new(),
            TokenInvertCaseMutator::new(),
            TokenStreamDictInsertMutator::new(max_tokens),
            TokenReplaceDictMutator::new(),
            TokenStreamScannerMutator::new(max_tokens),
            TokenConvertMutator::new(),
            TokenReplaceSpecialCharMutator::new()
        ),
        2
    );

    let mutator = StdScheduledMutator::with_max_stack_pow(
        tuple_list!(
            PacketDeleteMutator::new(1),
            PacketDuplicateMutator::new(16),
            PacketReorderMutator::new(),
            packet_mutator,
            InsertRandomPacketMutator::new(),
            InsertGeneratedPacketMutator::new(),
            PacketCrossoverInsertMutator::new(),
            PacketCrossoverReplaceMutator::new(),
            PacketRepeatMutator::new(16, 16)
        ),
        2
    );

    let mutational = StdPowerMutationalStage::new(mutator);
    //let mutational = StdMutationalStage::new(mutator);

    let scheduler = 
        StateAwareWeightedScheduler::new(&mut state, &edges_observer, Some(PowerSchedule::FAST), &state_observer)
        /* StdWeightedScheduler::with_schedule(&mut state, &edges_observer, Some(PowerSchedule::FAST)) */
    ;
    //let scheduler = RandScheduler::new();

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
    
    if let Some(replay) = &args.replay {
        let replay = Path::new(replay);
        
        if replay.is_dir() {
            let num_entries = std::fs::read_dir(replay)?.count();
            
            for (i, entry) in std::fs::read_dir(replay)?.enumerate() {
                let entry = entry?.path();
                
                if entry.is_file() && entry.file_name().unwrap().to_str().unwrap().starts_with("dragonfly-") {
                    println!("Replaying {}... ({}/{})", entry.display(), i + 1, num_entries);
                    
                    let input = DragonflyInput::<FTPPacket>::from_file(entry)?;
                    fuzzer.evaluate_input(&mut state, &mut executor, &mut mgr, input)?;
                }
            }
        } else {
            let input = DragonflyInput::<FTPPacket>::from_file(replay)?;
            fuzzer.evaluate_input(&mut state, &mut executor, &mut mgr, input)?;
        }
        
        if args.trace {
            let crashes = state.solutions().count();
            println!();
            println!("Crashes left: {}", crashes);
        }
    } else {
        assert!(!args.debug && !args.trace);
        
        /*
        /* Start with a single interaction */
        let input = DragonflyInput::new(
            vec![
                FTPPacket::Ctrl(TokenStream::builder().text("USER ftp\r\n").build()),
                FTPPacket::Sep,
                FTPPacket::Ctrl(TokenStream::builder().text("PASS x\r\n").build()),
                FTPPacket::Sep,
                FTPPacket::Ctrl(TokenStream::builder().text("CWD uploads\r\n").build()),
                FTPPacket::Sep,
                FTPPacket::Ctrl(TokenStream::builder().text("EPSV\r\n").build()),
                FTPPacket::Sep,
                FTPPacket::Ctrl(TokenStream::builder().text("STOR packetio.txt\r\n").build()),
                FTPPacket::Data(TokenStream::builder().blob("content").build()),
                FTPPacket::Data(TokenStream::builder().build()),
                FTPPacket::Sep,
                FTPPacket::Ctrl(TokenStream::builder().text("QUIT\r\n").build()),
                FTPPacket::Sep, 
            ]
        );
        */
        
        /* Start with an empty corpus */
        let input = DragonflyInput::new(
            vec![
                FTPPacket::Ctrl(TokenStream::builder().build()),
            ]
        );
        
        fuzzer.evaluate_input(&mut state, &mut executor, &mut mgr, input)?;

        #[cfg(debug_assertions)]
        fuzzer.fuzz_loop_for(&mut stages, &mut executor, &mut state, &mut mgr, 50)?;
        #[cfg(not(debug_assertions))]
        fuzzer.fuzz_loop(&mut stages, &mut executor, &mut state, &mut mgr)?;
    }
    
    /* Dump state graph */
    if let Some(ipsm) = &args.ipsm {
        let mut file = File::create(ipsm)?;
        state.get_stategraph()?.dump_dot(&mut file)?;
        file.flush()?;
    }
    
    Ok(())
}
