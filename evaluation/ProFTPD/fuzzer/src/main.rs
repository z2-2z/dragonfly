use serde::{Serialize, Deserialize};
use dragonfly::{
    tokens::{TokenStream, HasTokenStream},
    components::{
        Packet, DragonflyInput, PacketCopyMutator,
        PacketDeleteMutator, PacketRepeatMutator, 
        PacketSwapMutator, TokenStreamMutator,
        PacketContentMutator, DragonflyForkserverExecutor,
        DragonflyDebugExecutor, PacketCreator, PacketInsertionMutator,
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
    Error, EventConfig, Evaluator, Input, SimpleEventManager,
    InMemoryCorpus, HasRand, CanTrack,
};
use libafl_bolts::prelude::{
    current_nanos, UnixShMemProvider, shmem::{ShMemProvider, ShMem},
    AsMutSlice, StdRand, tuple_list, current_time, StdShMemProvider,
    Cores, Rand,
};
use std::time::Duration;
use std::path::PathBuf;

use mimalloc::MiMalloc;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

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
    
    Print {
        file: String,
    },
    
    Replay {
        #[arg(long)]
        gdb: bool,
        
        file: String,
    },
    
    GenerateCorpus {
        dir: String,
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

impl<S> PacketCreator<S> for FTPPacket
where
    S: HasRand,
{
    fn create_packets(state: &mut S) -> Vec<Self> {
        match state.rand_mut().below(59) {
            0 => vec![
                FTPPacket::Ctrl("USER x\r\n".parse().unwrap()),
            ],
            1 => vec![
                FTPPacket::Ctrl("PASS x\r\n".parse().unwrap()),
            ],
            2 => vec![
                FTPPacket::Ctrl("ACCT x\r\n".parse().unwrap()),
            ],
            3 => vec![
                FTPPacket::Ctrl("CWD x\r\n".parse().unwrap()),
            ],
            4 => vec![
                FTPPacket::Ctrl("CDUP\r\n".parse().unwrap()),
            ],
            5 => vec![
                FTPPacket::Ctrl("SMNT x\r\n".parse().unwrap()),
            ],
            6 => vec![
                FTPPacket::Ctrl("REIN\r\n".parse().unwrap()),
            ],
            7 => vec![
                FTPPacket::Ctrl("QUIT\r\n".parse().unwrap()),
            ],
            8 => vec![
                FTPPacket::Ctrl("PORT 0,0,0,0,0,0\r\n".parse().unwrap()),
            ],
            9 => vec![
                FTPPacket::Ctrl("EPRT |0|0.0.0.0|0|\r\n".parse().unwrap()),
            ],
            10 => vec![
                FTPPacket::Ctrl("PASV\r\n".parse().unwrap()),
            ],
            11 => vec![
                FTPPacket::Ctrl("EPSV\r\n".parse().unwrap()),
            ],
            12 => {
                let form_code = state.rand_mut().choose([" N", " T", " C", ""]);
                
                match state.rand_mut().below(4) {
                    0 => vec![
                        FTPPacket::Ctrl(format!("TYPE A{}\r\n", form_code).parse().unwrap()),
                    ],
                    1 => vec![
                        FTPPacket::Ctrl(format!("TYPE E{}\r\n", form_code).parse().unwrap()),
                    ],
                    2 => vec![
                        FTPPacket::Ctrl("TYPE I\r\n".parse().unwrap()),
                    ],
                    3 => vec![
                        FTPPacket::Ctrl("TYPE L 0\r\n".parse().unwrap()),
                    ],
                    _ => unreachable!(),
                }
            },
            13 => match state.rand_mut().below(3) {
                0 => vec![
                    FTPPacket::Ctrl("STRU F\r\n".parse().unwrap()),
                ],
                1 => vec![
                    FTPPacket::Ctrl("STRU R\r\n".parse().unwrap()),
                ],
                2 => vec![
                    FTPPacket::Ctrl("STRU P\r\n".parse().unwrap()),
                ],
                _ => unreachable!(),
            },
            14 => match state.rand_mut().below(3) {
                0 => vec![
                    FTPPacket::Ctrl("MODE S\r\n".parse().unwrap()),
                ],
                1 => vec![
                    FTPPacket::Ctrl("MODE B\r\n".parse().unwrap()),
                ],
                2 => vec![
                    FTPPacket::Ctrl("MODE C\r\n".parse().unwrap()),
                ],
                _ => unreachable!(),
            },
            15 => vec![
                FTPPacket::Ctrl("RETR x\r\n".parse().unwrap()),
            ],
            16 => vec![
                FTPPacket::Sep,
                FTPPacket::Ctrl("STOR x\r\n".parse().unwrap()),
                FTPPacket::Data,
                FTPPacket::Sep,
            ],
            17 => vec![
                FTPPacket::Ctrl("STOU\r\n".parse().unwrap()),
            ],
            18 => vec![
                FTPPacket::Sep,
                FTPPacket::Ctrl("APPE x\r\n".parse().unwrap()),
                FTPPacket::Data,
                FTPPacket::Sep,
            ],
            19 => match state.rand_mut().below(2) {
                0 => vec![
                    FTPPacket::Ctrl("ALLO 0\r\n".parse().unwrap()),
                ],
                1 => vec![
                    FTPPacket::Ctrl("ALLO 0 R 0\r\n".parse().unwrap()),
                ],
                _ => unreachable!(),
            },
            20 => vec![
                FTPPacket::Ctrl("REST x\r\n".parse().unwrap()),
            ],
            21 => vec![
                FTPPacket::Ctrl("RNFR x\r\n".parse().unwrap()),
            ],
            22 => vec![
                FTPPacket::Ctrl("RNTO x\r\n".parse().unwrap()),
            ],
            23 => vec![
                FTPPacket::Ctrl("ABOR\r\n".parse().unwrap()),
            ],
            24 => vec![
                FTPPacket::Ctrl("DELE x\r\n".parse().unwrap()),
            ],
            25 => vec![
                FTPPacket::Ctrl("MDTM x\r\n".parse().unwrap()),
            ],
            26 => vec![
                FTPPacket::Ctrl("RMD x\r\n".parse().unwrap()),
            ],
            27 => vec![
                FTPPacket::Ctrl("XRMD x\r\n".parse().unwrap()),
            ],
            28 => vec![
                FTPPacket::Ctrl("MKD x\r\n".parse().unwrap()),
            ],
            29 => vec![
                FTPPacket::Ctrl("MLST x\r\n".parse().unwrap()),
            ],
            30 => vec![
                FTPPacket::Ctrl("MLSD x\r\n".parse().unwrap()),
            ],
            31 => vec![
                FTPPacket::Ctrl("XMKD x\r\n".parse().unwrap()),
            ],
            32 => vec![
                FTPPacket::Ctrl("PWD\r\n".parse().unwrap()),
            ],
            33 => vec![
                FTPPacket::Ctrl("XPWD\r\n".parse().unwrap()),
            ],
            34 => vec![
                FTPPacket::Ctrl("SIZE x\r\n".parse().unwrap()),
            ],
            35 => vec![
                FTPPacket::Ctrl("LIST\r\n".parse().unwrap()),
            ],
            36 => vec![
                FTPPacket::Ctrl("NLST\r\n".parse().unwrap()),
            ],
            37 => vec![
                FTPPacket::Ctrl("SITE x\r\n".parse().unwrap()),
            ],
            38 => vec![
                FTPPacket::Ctrl("SYST\r\n".parse().unwrap()),
            ],
            39 => vec![
                FTPPacket::Ctrl("STAT x\r\n".parse().unwrap()),
            ],
            40 => vec![
                FTPPacket::Ctrl("FEAT\r\n".parse().unwrap()),
            ],
            41 => vec![
                FTPPacket::Ctrl("OPTS x\r\n".parse().unwrap()),
            ],
            42 => vec![
                FTPPacket::Ctrl("LANG x-x-x\r\n".parse().unwrap()),
            ],
            43 => vec![
                FTPPacket::Ctrl("ADAT eA==\r\n".parse().unwrap()),
            ],
            44 => vec![
                FTPPacket::Ctrl("AUTH x\r\n".parse().unwrap()),
            ],
            45 => vec![
                FTPPacket::Ctrl("CCC\r\n".parse().unwrap()),
            ],
            46 => vec![
                FTPPacket::Ctrl("CONF eA==\r\n".parse().unwrap()),
            ],
            47 => vec![
                FTPPacket::Ctrl("ENC eA==\r\n".parse().unwrap()),
            ],
            48 => vec![
                FTPPacket::Ctrl("MIC eA==\r\n".parse().unwrap()),
            ],
            49 => vec![
                FTPPacket::Ctrl("PBSZ 0\r\n".parse().unwrap()),
            ],
            50 => match state.rand_mut().below(4) {
                0 => vec![
                    FTPPacket::Ctrl("PROT C\r\n".parse().unwrap()),
                ],
                1 => vec![
                    FTPPacket::Ctrl("PROT S\r\n".parse().unwrap()),
                ],
                2 => vec![
                    FTPPacket::Ctrl("PROT E\r\n".parse().unwrap()),
                ],
                3 => vec![
                    FTPPacket::Ctrl("PROT P\r\n".parse().unwrap()),
                ],
                _ => unreachable!(),
            },
            51 => vec![
                FTPPacket::Ctrl("MFF x\r\n".parse().unwrap()),
            ],
            52 => vec![
                FTPPacket::Ctrl("MFMT 0 x\r\n".parse().unwrap()),
            ],
            53 => vec![
                FTPPacket::Ctrl("HOST x\r\n".parse().unwrap()),
            ],
            54 => {
                let name = state.rand_mut().choose(["Version", "Name", "Vendor"]);
                vec![
                    FTPPacket::Ctrl(format!("CSID {}=x;\r\n", name).parse().unwrap()),
                ]
            },
            55 => vec![
                FTPPacket::Ctrl("CLNT\r\n".parse().unwrap()),
            ],
            56 => vec![
                FTPPacket::Ctrl("RANG\r\n".parse().unwrap()),
            ],
            57 => vec![
                FTPPacket::Sep
            ],
            58 => vec![
                FTPPacket::Data
            ],
            _ => unreachable!(),
        }
    }
}

fn fuzz(output: String, corpus: Option<String>, debug_child: bool, cores: String) {
    let mut run_client = |state: Option<_>, mut mgr: LlmpRestartingEventManager<_, _, _>, _core_id| {
        let timeout = Duration::from_millis(10000);
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
        
        let edges_observer = HitcountsMapObserver::new(unsafe { StdMapObserver::new("shared_mem", shmem_buf) }).track_indices();
        let time_observer = TimeObserver::new("time");
        
        let map_feedback = MaxMapFeedback::new(&edges_observer);
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
            PacketContentMutator::new(TokenStreamMutator::new(128)),
            PacketInsertionMutator::new()
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
            .program("./proftpd-fuzzing")
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
                        FTPPacket::Ctrl("".parse().unwrap()),
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

fn print(file: String) {
    let input = DragonflyInput::<FTPPacket>::from_file(file).unwrap();
    
    for packet in input.packets() {
        match packet {
            FTPPacket::Ctrl(stream) => {
                let mut buf = vec![0; stream.serialized_len()];
                let len = stream.serialize_into_buffer(&mut buf);
                let str = std::str::from_utf8(&buf[..len]).unwrap();
                println!("ctrl: {:?}", str);
            },
            FTPPacket::Data => println!("<data>"),
            FTPPacket::Sep => println!("<sep>"),
        }
    }
}

fn replay(file: String, gdb: bool) {
    let timeout = Duration::from_secs(999999);
    let signal = str::parse::<Signal>("SIGKILL").unwrap();
    let input = DragonflyInput::<FTPPacket>::from_file(file).unwrap();
    
    let monitor = NopMonitor::new();
    let mut mgr = SimpleEventManager::new(monitor);
    
    let mut objective = feedback_or!(
        CrashFeedback::new(),
        TimeoutFeedback::new()
    );
    
    let mut state = StdState::new(
        StdRand::with_seed(1234),
        InMemoryCorpus::<DragonflyInput<FTPPacket>>::new(),
        InMemoryCorpus::<DragonflyInput<FTPPacket>>::new(),
        &mut (),
        &mut objective,
    ).unwrap();
    
    let scheduler = QueueScheduler::new();
    
    let mut fuzzer = StdFuzzer::new(scheduler, (), objective);
    
    let mut shmem_provider = UnixShMemProvider::new().unwrap();
    
    if gdb {
        let mut executor = DragonflyDebugExecutor::new(&mut shmem_provider).unwrap();
        executor.arg("/usr/bin/gdb");
        executor.arg("-ex");
        executor.arg("set environment LD_PRELOAD ./libdragonfly.so");
        executor.arg("--args");
        executor.arg("./proftpd-debug");
        executor.arg("-d");
        executor.arg("10");
        executor.arg("-q");
        executor.arg("-X");
        executor.arg("-c");
        executor.arg("/proftpd/config");
        executor.arg("-n");
        
        fuzzer.evaluate_input(
            &mut state,
            &mut executor,
            &mut mgr,
            input,
        ).unwrap();
    } else {
        let mut executor = DragonflyForkserverExecutor::builder()
            .observers(tuple_list!())
            .shmem_provider(&mut shmem_provider)
            .timeout(timeout)
            .signal(signal)
            .debug_child(true)
            .is_deferred_forkserver(true)
            .env("LD_PRELOAD", "./libdragonfly.so")
            .program("./proftpd-fuzzing")
            .args(["-d", "10", "-q", "-X", "-c", "/proftpd/config", "-n"])
            .build()
            .unwrap();
        
        fuzzer.evaluate_input(
            &mut state,
            &mut executor,
            &mut mgr,
            input,
        ).unwrap();
    }
}

fn generate_corpus(dir: String) {
    DragonflyInput::new(
        vec![
            FTPPacket::Ctrl("USER ftp\r\n".parse().unwrap()),
            FTPPacket::Sep,
            FTPPacket::Ctrl("PASS fuck@you.org\r\n".parse().unwrap()),
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
    ).to_file(format!("{}/anon-store", dir)).unwrap();
    DragonflyInput::new(
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
    ).to_file(format!("{}/user-store", dir)).unwrap();
    DragonflyInput::new(
        vec![
            FTPPacket::Ctrl("USER user\r\n".parse().unwrap()),
            FTPPacket::Ctrl("PASS user\r\n".parse().unwrap()),
            FTPPacket::Ctrl("PORT 127,0,0,1,80,80\r\n".parse().unwrap()),
            FTPPacket::Ctrl("LIST\r\n".parse().unwrap()),
            FTPPacket::Ctrl("NOOP\r\n".parse().unwrap()),
            FTPPacket::Ctrl("QUIT\r\n".parse().unwrap()),
        ]
    ).to_file(format!("{}/list-root", dir)).unwrap();
    DragonflyInput::new(
        vec![
            FTPPacket::Ctrl("USER ftp\r\n".parse().unwrap()),
            FTPPacket::Ctrl("PASS fuck@you.org\r\n".parse().unwrap()),
            FTPPacket::Ctrl("PASV\r\n".parse().unwrap()),
            FTPPacket::Ctrl("LIST dir -1AaBCcdFhLlnRrStUu\r\n".parse().unwrap()),
            FTPPacket::Ctrl("REIN\r\n".parse().unwrap()),
            FTPPacket::Ctrl("USER ftp\r\n".parse().unwrap()),
            FTPPacket::Ctrl("PASS fuck@you.org\r\n".parse().unwrap()),
            FTPPacket::Ctrl("EPRT |1|132.235.1.2|6275|\r\n".parse().unwrap()),
            FTPPacket::Ctrl("LIST dir/empty -1AaBCcdFhLlnRrStUu\r\n".parse().unwrap()),
            FTPPacket::Ctrl("ABOR\r\n".parse().unwrap()),
        ]
    ).to_file(format!("{}/rein-abor", dir)).unwrap();
    DragonflyInput::new(
        vec![
            FTPPacket::Ctrl("USER user\r\n".parse().unwrap()),
            FTPPacket::Ctrl("PASS user\r\n".parse().unwrap()),
            FTPPacket::Ctrl("PASV\r\n".parse().unwrap()),
            FTPPacket::Ctrl("RETR file\r\n".parse().unwrap()),
            FTPPacket::Ctrl("NOOP\r\n".parse().unwrap()),
            FTPPacket::Ctrl("QUIT\r\n".parse().unwrap()),
        ]
    ).to_file(format!("{}/retr-file", dir)).unwrap();
    //TODO: rename file
}

fn main() {
    let args = Args::parse();

    match args.command {
        Subcommand::Fuzz { output, corpus, debug, cores } => fuzz(output, corpus, debug, cores),
        Subcommand::Print { file } => print(file),
        Subcommand::Replay { file, gdb } => replay(file, gdb),
        Subcommand::GenerateCorpus { dir } => generate_corpus(dir),
    }
}
