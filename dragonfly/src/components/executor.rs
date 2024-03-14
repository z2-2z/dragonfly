use libafl_bolts::prelude::{
    AsMutSlice,
    shmem::{ShMem, ShMemProvider},
};
use libafl::prelude::{
    Error,
    Executor,
    ExitKind,
    Forkserver,
    HasObservers,
    ObserversTuple,
    UsesInput,
    UsesObservers,
    UsesState,
    State,
    HasExecutions,
};
use nix::{
    sys::{
        signal::{
            kill,
            Signal,
        },
        time::{
            TimeSpec,
            TimeValLike,
        },
    },
    unistd::Pid,
};
use std::{
    ffi::{
        OsStr,
        OsString,
    },
    marker::PhantomData,
    time::Duration,
};

use crate::components::{
    DragonflyInput, Packet,
};

pub const PACKET_CHANNEL_SIZE: usize = 16 * 1024 * 1024;
const PACKET_CHANNEL_ENV_VAR: &str = "__LIBDRAGONFLY_PACKET_CHANNEL";

#[derive(Debug)]
pub struct DragonflyForkserverExecutor<OT, S, SP, P>
where
    OT: ObserversTuple<S>,
    S:  State + UsesInput<Input = DragonflyInput<P>>,
    SP: ShMemProvider,
    P: Packet,
{
    observers: OT,
    packet_channel: SP::ShMem,
    timeout: TimeSpec,
    signal: Signal,
    forkserver: Forkserver,
    phantom: PhantomData<S>,
}

impl<'a, OT, S, SP, P> DragonflyForkserverExecutor<OT, S, SP, P>
where
    OT: ObserversTuple<S>,
    S:  State + UsesInput<Input = DragonflyInput<P>>,
    SP: ShMemProvider,
    P: Packet,
{
    fn new(observers: OT, packet_channel: SP::ShMem, timeout: TimeSpec, signal: Signal, forkserver: Forkserver) -> Self {
        Self {
            observers,
            packet_channel,
            timeout,
            signal,
            forkserver,
            phantom: PhantomData,
        }
    }
    
    pub fn builder() -> DragonflyForkserverExecutorBuilder<'a, OT, S, SP, P> {
        DragonflyForkserverExecutorBuilder::new()
    }
}

impl<OT, S, SP, P> UsesState for DragonflyForkserverExecutor<OT, S, SP, P>
where
    OT: ObserversTuple<S>,
    S:  State + UsesInput<Input = DragonflyInput<P>>,
    SP: ShMemProvider,
    P: Packet,
{
    type State = S;
}

impl<OT, S, SP, P> UsesObservers for DragonflyForkserverExecutor<OT, S, SP, P>
where
    OT: ObserversTuple<S>,
    S:  State + UsesInput<Input = DragonflyInput<P>>,
    SP: ShMemProvider,
    P: Packet,
{
    type Observers = OT;
}

impl<OT, S, SP, P> HasObservers for DragonflyForkserverExecutor<OT, S, SP, P>
where
    OT: ObserversTuple<S>,
    S:  State + UsesInput<Input = DragonflyInput<P>>,
    SP: ShMemProvider,
    P: Packet,
{
    fn observers(&self) -> &OT {
        &self.observers
    }

    fn observers_mut(&mut self) -> &mut OT {
        &mut self.observers
    }
}

impl<OT, S, SP, P, EM, Z> Executor<EM, Z> for DragonflyForkserverExecutor<OT, S, SP, P>
where
    OT: ObserversTuple<S>,
    S:  State + UsesInput<Input = DragonflyInput<P>> + HasExecutions,
    SP: ShMemProvider,
    P: Packet,
    EM: UsesState<State = S>,
    Z: UsesState<State = S>,
{
    fn run_target(&mut self, _fuzzer: &mut Z, state: &mut S, _mgr: &mut EM, input: &DragonflyInput<P>) -> Result<ExitKind, Error> {
        *state.executions_mut() += 1;
        
        /* Serialize input into packet channel */
        let buffer = self.packet_channel.as_mut_slice();
        input.serialize_dragonfly_format(buffer);

        /* Launch the client */
        let mut exit_kind = ExitKind::Ok;
        let last_run_timed_out = self.forkserver.last_run_timed_out();
        let send_len = self.forkserver.write_ctl(last_run_timed_out as i32)?;
        self.forkserver.set_last_run_timed_out(false);

        if send_len != 4 {
            return Err(Error::unknown("Unable to request new process from fork server (OOM?)"));
        }

        let (recv_pid_len, pid) = self.forkserver.read_st()?;

        if recv_pid_len != 4 {
            return Err(Error::unknown("Unable to request new process from fork server (OOM?)"));
        }
        if pid <= 0 {
            return Err(Error::unknown("Fork server is misbehaving (OOM?)"));
        }

        self.forkserver.set_child_pid(Pid::from_raw(pid));

        if let Some(status) = self.forkserver.read_st_timed(&self.timeout)? {
            self.forkserver.set_status(status);

            if libc::WIFSIGNALED(self.forkserver.status()) {
                exit_kind = ExitKind::Crash;
            }
        } else {
            self.forkserver.set_last_run_timed_out(true);
            let _ = kill(self.forkserver.child_pid(), self.signal);
            let (recv_status_len, _) = self.forkserver.read_st()?;
            if recv_status_len != 4 {
                return Err(Error::unknown("Could not kill timed-out child"));
            }
            exit_kind = ExitKind::Timeout;
        }

        self.forkserver.set_child_pid(Pid::from_raw(0));

        Ok(exit_kind)
    }
}

pub struct DragonflyForkserverExecutorBuilder<'a, OT, S, SP, P>
where
    OT: ObserversTuple<S>,
    S:  State + UsesInput<Input = DragonflyInput<P>>,
    SP: ShMemProvider,
    P: Packet,
{
    shmem_provider: Option<&'a mut SP>,
    observers: Option<OT>,
    signal: Signal,
    timeout: Option<Duration>,
    program: Option<OsString>,
    arguments: Vec<OsString>,
    envs: Vec<(OsString, OsString)>,
    is_deferred: bool,
    debug_child: bool,
    phantom: PhantomData<(S, P)>,
}

impl<'a, OT, S, SP, P> DragonflyForkserverExecutorBuilder<'a, OT, S, SP, P>
where
    OT: ObserversTuple<S>,
    S:  State + UsesInput<Input = DragonflyInput<P>>,
    SP: ShMemProvider,
    P: Packet,
{
    #[allow(clippy::new_without_default)]
    fn new() -> Self {
        Self {
            shmem_provider: None,
            observers: None,
            signal: Signal::SIGKILL,
            timeout: None,
            program: None,
            arguments: Vec::new(),
            envs: Vec::new(),
            is_deferred: false,
            debug_child: false,
            phantom: PhantomData,
        }
    }

    pub fn shmem_provider(mut self, provider: &'a mut SP) -> Self {
        self.shmem_provider = Some(provider);
        self
    }

    pub fn observers(mut self, observers: OT) -> Self {
        self.observers = Some(observers);
        self
    }

    pub fn signal(mut self, signal: Signal) -> Self {
        self.signal = signal;
        self
    }

    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    pub fn program<O: AsRef<OsStr>>(mut self, program: O) -> Self {
        self.program = Some(program.as_ref().to_owned());
        self
    }

    pub fn arg<O: AsRef<OsStr>>(mut self, arg: O) -> Self {
        self.arguments.push(arg.as_ref().to_owned());
        self
    }

    pub fn args<IT, O>(mut self, args: IT) -> Self
    where
        IT: IntoIterator<Item = O>,
        O: AsRef<OsStr>,
    {
        for arg in args {
            self.arguments.push(arg.as_ref().to_owned());
        }
        self
    }

    pub fn env<K, V>(mut self, key: K, val: V) -> Self
    where
        K: AsRef<OsStr>,
        V: AsRef<OsStr>,
    {
        self.envs.push((key.as_ref().to_owned(), val.as_ref().to_owned()));
        self
    }

    pub fn envs<IT, K, V>(mut self, vars: IT) -> Self
    where
        IT: IntoIterator<Item = (K, V)>,
        K: AsRef<OsStr>,
        V: AsRef<OsStr>,
    {
        for (key, val) in vars {
            self.envs.push((key.as_ref().to_owned(), val.as_ref().to_owned()));
        }
        self
    }

    pub fn is_deferred_forkserver(mut self, is_deferred: bool) -> Self {
        self.is_deferred = is_deferred;
        self
    }

    pub fn debug_child(mut self, debug_child: bool) -> Self {
        self.debug_child = debug_child;
        self
    }

    pub fn build(self) -> Result<DragonflyForkserverExecutor<OT, S, SP, P>, Error> {
        macro_rules! get_value {
            ($name:ident) => {
                self.$name.ok_or(Error::illegal_argument(format!("DragonflyExecutorBuilder: {} was not set", stringify!($name))))?
            };
        }

        let shmem_provider = get_value!(shmem_provider);
        let observers = get_value!(observers);
        let timeout = get_value!(timeout);
        let program = get_value!(program);

        let packet_channel = shmem_provider.new_shmem(PACKET_CHANNEL_SIZE)?;
        packet_channel.write_to_env(PACKET_CHANNEL_ENV_VAR)?;

        let timeout = TimeSpec::milliseconds(timeout.as_millis() as i64);

        let mut forkserver = Forkserver::new(program, self.arguments, self.envs, -1, false, 0, false, self.is_deferred, self.debug_child)?;

        // Initial forkserver handshake
        let (rlen, _) = forkserver.read_st()?;

        if rlen != 4 {
            return Err(Error::unknown("Failed to start a forkserver"));
        }

        Ok(DragonflyForkserverExecutor::new(observers, packet_channel, timeout, self.signal, forkserver))
    }
}
