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
    InMemoryCorpus, 
    StdScheduledMutator, StdMutationalStage, QueueScheduler,
    StdFuzzer, Fuzzer, SimplePrintingMonitor, Launcher,
    Error, EventConfig, Evaluator,
};
use libafl_bolts::prelude::{
    current_nanos, UnixShMemProvider, shmem::{ShMemProvider, ShMem},
    AsMutSlice, StdRand, tuple_list, StdShMemProvider,
    Cores,
};
use std::time::Duration;

#[derive(clap::Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(long, default_value_t = String::from("0"))]
    cores: String,
    
    cmd: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Hash)]
struct GenericPacket {
    content: TokenStream,
    connection: usize,
    terminates_group: bool,
}

impl Packet for GenericPacket {
    fn serialize_content(&self, buffer: &mut [u8]) -> Option<usize> {
        Some(self.content.serialize_into_buffer(buffer))
    }
    
    fn connection(&self) -> usize {
        self.connection
    }
    
    fn terminates_group(&self) -> bool {
        self.terminates_group
    }
}

impl HasTokenStream for GenericPacket {
    fn has_token_stream(&self) -> bool {
        true
    }
    
    fn token_stream(&self) -> &TokenStream {
        &self.content
    }
    
    fn token_stream_mut(&mut self) -> &mut TokenStream {
        &mut self.content
    }
}

fn fuzz(cores: String, mut cmd: Vec<String>) {
    let mut run_client = |state: Option<_>, mut mgr: LlmpRestartingEventManager<_, _>, _core_id| {
        let timeout = Duration::from_millis(5000);
        let signal = str::parse::<Signal>("SIGKILL").unwrap();
        let seed = current_nanos();
        
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
                InMemoryCorpus::<DragonflyInput<GenericPacket>>::new(),
                InMemoryCorpus::<DragonflyInput<GenericPacket>>::new(),
                &mut feedback,
                &mut objective,
            )?
        };
        
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
        
        let program = cmd.remove(0);
        let mut builder = DragonflyForkserverExecutor::builder()
            .observers(tuple_list!(edges_observer, time_observer))
            .shmem_provider(&mut shmem_provider)
            .timeout(timeout)
            .signal(signal)
            .debug_child(true)
            .program(program)
            .args(&cmd)
            .is_deferred_forkserver(true);
        
        if let Ok(value) = std::env::var("PRELOAD") {
            builder = builder.env("LD_PRELOAD", value);
        }
        
        let mut executor = builder.build()?;
        
        if state.must_load_initial_inputs() {
            let input = DragonflyInput::new(
                vec![
                    GenericPacket {
                        content: "TEST123".parse().unwrap(),
                        connection: 0,
                        terminates_group: true,
                    },
                    GenericPacket {
                        content: "hello world".parse().unwrap(),
                        connection: 0,
                        terminates_group: true,
                    },
                ]
            );
            
            fuzzer.add_input(&mut state, &mut executor, &mut mgr, input)?;
        }
        
        fuzzer.fuzz_loop(&mut stages, &mut executor, &mut state, &mut mgr)?;
        Ok(())
    };
    
    let monitor = SimplePrintingMonitor::new();
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
    fuzz(args.cores, args.cmd);
}
