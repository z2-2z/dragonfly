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
    unistd::{Pid, execve},
};
use std::{
    ffi::{
        OsStr,
        OsString,
    },
    marker::PhantomData,
    time::Duration,
};
use std::ffi::CString;
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
        let last_run_timed_out = self.forkserver.last_run_timed_out_raw();
        let send_len = self.forkserver.write_ctl(last_run_timed_out)?;
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
        
        if !libc::WIFSTOPPED(self.forkserver.status()) {
            self.forkserver.reset_child_pid();
        }

        Ok(exit_kind)
    }
}

fn do_forkserver_handshake(forkserver: &mut Forkserver) -> Result<(), Error> {
    const FS_NEW_OPT_MAPSIZE: i32 = 0x00000001;
    const FS_NEW_OPT_AUTODICT: i32 = 0x00000800;
    
    /* Echo back version */
    let (rlen, version) = forkserver.read_st()?;

    if rlen != 4 {
        return Err(Error::unknown("Failed to receive forkserver version"));
    }
    
    forkserver.write_ctl(!version)?;
    
    /* Receive options */
    let (rlen, options) = forkserver.read_st()?;

    if rlen != 4 {
        return Err(Error::unknown("Failed to receive forkserver options"));
    }
    
    if (options & FS_NEW_OPT_MAPSIZE) != 0 {
        let (rlen, _) = forkserver.read_st()?;

        if rlen != 4 {
            return Err(Error::unknown("Failed to receive forkserver map_size"));
        }
    }
    
    if (options & FS_NEW_OPT_AUTODICT) != 0 {
        let (rlen, dict_len) = forkserver.read_st()?;

        if rlen != 4 {
            return Err(Error::unknown("Failed to receive autodict length"));
        }
        
        let (rlen, _) = forkserver.read_st_size(dict_len as usize)?;
        
        if rlen != dict_len as usize {
            return Err(Error::unknown("Failed to receive autodict"));
        }
    }
    
    /* Receive welcome message */
    let (rlen, _) = forkserver.read_st()?;

    if rlen != 4 {
        return Err(Error::unknown("Failed to recieve forkserver welcome message"));
    }
    
    Ok(())
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
        do_forkserver_handshake(&mut forkserver)?;

        Ok(DragonflyForkserverExecutor::new(observers, packet_channel, timeout, self.signal, forkserver))
    }
}

#[derive(Debug)]
pub struct DragonflyDebugExecutor<S, SP, P>
where
    S:  State + UsesInput<Input = DragonflyInput<P>>,
    SP: ShMemProvider,
    P: Packet,
{
    observers: (),
    packet_channel: SP::ShMem,
    args: Vec<CString>,
    envs: Vec<CString>,
    phantom: PhantomData<S>,
}

impl<S, SP, P> DragonflyDebugExecutor<S, SP, P>
where
    S:  State + UsesInput<Input = DragonflyInput<P>>,
    SP: ShMemProvider,
    P: Packet,
{
    pub fn new(shmem_provider: &mut SP) -> Result<Self, Error> {
        let packet_channel = shmem_provider.new_shmem(PACKET_CHANNEL_SIZE)?;
        
        let mut ret = Self {
            observers: (),
            packet_channel,
            args: Vec::new(),
            envs: Vec::new(),
            phantom: PhantomData,
        };
        ret.inherit_env();
        Ok(ret)
    }
    
    fn inherit_env(&mut self) {
        for (key, value) in std::env::vars() {
            self.env(key, value);
        }
    }
    
    pub fn arg<T: Into<Vec<u8>>>(&mut self, arg: T) {
        let arg = CString::new(arg).unwrap();
        self.args.push(arg);
    }
    
    pub fn env<T1: Into<Vec<u8>>, T2: Into<Vec<u8>>>(&mut self, key: T1, value: T2) {
        let key = key.into();
        let value = value.into();
        
        let mut env_str = vec![0; key.len() + 1 + value.len()];
        env_str[..key.len()].copy_from_slice(&key);
        env_str[key.len()] = b'=';
        env_str[key.len() + 1..].copy_from_slice(&value);
        
        let env_str = CString::new(env_str).unwrap();
        self.envs.push(env_str);
    }
}

impl<S, SP, P> UsesState for DragonflyDebugExecutor<S, SP, P>
where
    S:  State + UsesInput<Input = DragonflyInput<P>>,
    SP: ShMemProvider,
    P: Packet,
{
    type State = S;
}

impl<S, SP, P> UsesObservers for DragonflyDebugExecutor<S, SP, P>
where
    S:  State + UsesInput<Input = DragonflyInput<P>>,
    SP: ShMemProvider,
    P: Packet,
{
    type Observers = ();
}

impl<S, SP, P> HasObservers for DragonflyDebugExecutor<S, SP, P>
where
    S:  State + UsesInput<Input = DragonflyInput<P>>,
    SP: ShMemProvider,
    P: Packet,
{
    fn observers(&self) -> &() {
        &self.observers
    }

    fn observers_mut(&mut self) -> &mut () {
        &mut self.observers
    }
}

impl<S, SP, P, EM, Z> Executor<EM, Z> for DragonflyDebugExecutor<S, SP, P>
where
    S:  State + UsesInput<Input = DragonflyInput<P>> + HasExecutions,
    SP: ShMemProvider,
    P: Packet,
    EM: UsesState<State = S>,
    Z: UsesState<State = S>,
{
    fn run_target(&mut self, _fuzzer: &mut Z, _state: &mut S, _mgr: &mut EM, input: &DragonflyInput<P>) -> Result<ExitKind, Error> {
        self.env(PACKET_CHANNEL_ENV_VAR, self.packet_channel.id().as_str());
        
        let buffer = self.packet_channel.as_mut_slice();
        input.serialize_dragonfly_format(buffer);
        
        let program = self.args[0].clone();
        execve(
            &program,
            &self.args,
            &self.envs,
        ).unwrap();

        unreachable!()
    }
}
