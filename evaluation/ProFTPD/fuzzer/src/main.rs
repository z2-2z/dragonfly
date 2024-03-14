use serde::{Serialize, Deserialize};
use dragonfly::{
    tokens::{TokenStream, HasTokenStream},
    components::{
        Packet, DragonflyInput, PacketCopyMutator,
        PacketDeleteMutator, PacketRepeatMutator, 
        PacketSwapMutator, TokenStreamMutator,
        PacketContentMutator, DragonflyForkserverExecutor,
    },
};
use clap::Parser;
use nix::sys::signal::Signal;
use libafl::prelude::{
    LlmpRestartingEventManager, HitcountsMapObserver, StdMapObserver,
    TimeObserver, MaxMapFeedback, TimeFeedback, CalibrationStage,
    feedback_or, CrashFeedback, TimeoutFeedback, StdState,
    CachedOnDiskCorpus, OnDiskCorpus, Tokens, HasMetadata,
    StdScheduledMutator, StdMutationalStage, QueueScheduler,
    StdFuzzer, Fuzzer, OnDiskJSONMonitor, NopMonitor, Launcher,
    Error, EventConfig, Evaluator,
};
use libafl_bolts::prelude::{
    current_nanos, UnixShMemProvider, shmem::{ShMemProvider, ShMem},
    AsMutSlice, StdRand, tuple_list, current_time, StdShMemProvider,
    Cores,
};
use std::time::Duration;
use std::path::PathBuf;

#[derive(clap::Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Subcommand,
}

#[derive(clap::Subcommand)]
enum Subcommand {
    Fuzz {
        #[arg(long)]
        output: String,
        
        #[arg(long)]
        corpus: Option<String>,
        
        #[arg(long)]
        debug: bool,
        
        #[arg(long, default_value_t = String::from("0"))]
        cores: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, Hash)]
enum FTPPacket {
    Ctrl(TokenStream),
    Data,
    Sep,
}

impl Packet for FTPPacket {
    fn serialize_content(&self, buffer: &mut [u8]) -> Option<usize> {
        match self {
            FTPPacket::Ctrl(stream) => Some(stream.serialize_into_buffer(buffer)),
            FTPPacket::Data => {
                const PLACEHOLDER: &[u8] = b"data";
                let len = std::cmp::min(PLACEHOLDER.len(), buffer.len());
                buffer[..len].copy_from_slice(&PLACEHOLDER[..len]);
                Some(len)
            },
            FTPPacket::Sep => None,
        }
    }
    
    fn connection(&self) -> usize {
        match self {
            FTPPacket::Ctrl(_) => 0,
            FTPPacket::Data => 1,
            FTPPacket::Sep => unreachable!(),
        }
    }
    
    fn terminates_group(&self) -> bool {
        match self {
            FTPPacket::Ctrl(_) => false,
            FTPPacket::Data => false,
            FTPPacket::Sep => true,
        }
    }
}

impl HasTokenStream for FTPPacket {
    fn has_token_stream(&self) -> bool {
        matches!(self, FTPPacket::Ctrl(_))
    }
    
    fn token_stream(&self) -> &TokenStream {
        match self {
            FTPPacket::Ctrl(stream) => stream,
            _ => unreachable!(),
        }
    }
    
    fn token_stream_mut(&mut self) -> &mut TokenStream {
        match self {
            FTPPacket::Ctrl(stream) => stream,
            _ => unreachable!(),
        }
    }
}

fn fuzz(output: String, corpus: Option<String>, debug_child: bool, cores: String) {
    let mut run_client = |state: Option<_>, mut mgr: LlmpRestartingEventManager<_, _>, _core_id| {
        let timeout = Duration::from_millis(5000);
        let signal = str::parse::<Signal>("SIGKILL").unwrap();
        let seed = current_nanos();
        let loglevel = if debug_child {
            "10"
        } else {
            "0"
        };
        
        let mut shmem_provider = UnixShMemProvider::new()?;
        const MAP_SIZE: usize = 65536;
        let mut shmem = shmem_provider.new_shmem(MAP_SIZE)?;
        shmem.write_to_env("__AFL_SHM_ID")?;
        let shmem_buf = shmem.as_mut_slice();
        std::env::set_var("AFL_MAP_SIZE", format!("{}", MAP_SIZE));
        
        let edges_observer = HitcountsMapObserver::new(unsafe { StdMapObserver::new("shared_mem", shmem_buf) });
        let time_observer = TimeObserver::new("time");
        
        let map_feedback = MaxMapFeedback::tracking(&edges_observer, true, false);
        let time_feedback = TimeFeedback::with_observer(&time_observer);
        
        let calibration = CalibrationStage::new(&map_feedback);
        
        let mut feedback = feedback_or!(
            map_feedback,
            time_feedback
        );
        
        let mut objective = feedback_or!(
            CrashFeedback::new(),
            TimeoutFeedback::new()
        );
        
        let mut state = if let Some(state) = state { 
            state
         } else {
            StdState::new(
                StdRand::with_seed(seed),
                CachedOnDiskCorpus::<DragonflyInput<FTPPacket>>::new(format!("{}/queue", &output), 128)?,
                OnDiskCorpus::<DragonflyInput<FTPPacket>>::new(format!("{}/crashes", &output))?,
                &mut feedback,
                &mut objective,
            )?
        };
        
        let dictionary = Tokens::from_file("./ftp.dict")?;
        state.add_metadata(dictionary);
        
        let max_packets = 16;
        let mutators = tuple_list!(
            PacketCopyMutator::new(max_packets),
            PacketDeleteMutator::new(0),
            PacketRepeatMutator::new(max_packets),
            PacketSwapMutator::new(),
            PacketContentMutator::new(TokenStreamMutator::new(128))
        );
        let mutator = StdScheduledMutator::with_max_stack_pow(mutators, 2);
        
        let mut stages = tuple_list!(calibration, StdMutationalStage::new(mutator));
        
        let scheduler = QueueScheduler::new();
        
        let mut fuzzer = StdFuzzer::new(scheduler, feedback, objective);
        
        let mut executor = DragonflyForkserverExecutor::builder()
            .observers(tuple_list!(edges_observer, time_observer))
            .shmem_provider(&mut shmem_provider)
            .timeout(timeout)
            .signal(signal)
            .debug_child(debug_child)
            .env("LD_PRELOAD", "./libdragonfly.so")
            .program("./proftpd")
            .args(["-d", loglevel, "-q", "-X", "-c", "/proftpd/config", "-n"])
            .is_deferred_forkserver(true)
            .build()?;
        
        if state.must_load_initial_inputs() {
            if let Some(corpus) = &corpus {
                state.load_initial_inputs(&mut fuzzer, &mut executor, &mut mgr, &[
                    PathBuf::from(corpus),
                ])?;
            } else {
                let input = DragonflyInput::new(
                    vec![
                        FTPPacket::Ctrl("USER user\r\n".parse().unwrap()),
                        FTPPacket::Sep,
                        FTPPacket::Ctrl("PASS user\r\n".parse().unwrap()),
                        FTPPacket::Sep,
                        FTPPacket::Ctrl("CWD uploads\r\n".parse().unwrap()),
                        FTPPacket::Sep,
                        FTPPacket::Ctrl("EPSV\r\n".parse().unwrap()),
                        FTPPacket::Sep,
                        FTPPacket::Ctrl("STOR packetio.txt\r\n".parse().unwrap()),
                        FTPPacket::Data,
                        FTPPacket::Sep,
                        FTPPacket::Ctrl("QUIT\r\n".parse().unwrap()),
                    ]
                );
                
                fuzzer.evaluate_input(&mut state, &mut executor, &mut mgr, input)?;
            }
        }
        
        fuzzer.fuzz_loop(&mut stages, &mut executor, &mut state, &mut mgr)?;
        Ok(())
    };
    
    let mut last_updated = 0;
    let monitor = OnDiskJSONMonitor::new(
        format!("{}/stats.jsonl", &output),
        NopMonitor::new(),
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
    let shmem_provider = StdShMemProvider::new().unwrap();
    let cores = Cores::from_cmdline(&cores).unwrap();

    match Launcher::builder()
        .shmem_provider(shmem_provider)
        .configuration(EventConfig::AlwaysUnique)
        .monitor(monitor)
        .run_client(&mut run_client)
        .cores(&cores)
        .build()
        .launch()
    {
        Err(Error::ShuttingDown) | Ok(()) => {},
        e => panic!("{:?}", e),
    }
}

fn main() {
    let args = Args::parse();

    match args.command {
        Subcommand::Fuzz { output, corpus, debug, cores } => fuzz(output, corpus, debug, cores),
    }
}
