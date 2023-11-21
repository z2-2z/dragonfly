use libafl_bolts::{
    AsMutSlice,
    shmem::{ShMem, ShMemProvider},
};
use libafl::prelude::{
    Error,
    Executor,
    ExitKind,
    Forkserver,
    HasObservers,
    Input,
    ObserversTuple,
    UsesInput,
    UsesObservers,
    UsesState,
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

use crate::input::{
    HasPacketVector,
    SerializeIntoBuffer,
};

#[derive(Clone, Copy)]
#[repr(u32)]
enum PacketType {
    Data = 1,
    Sep = 2,
    Eof = 3,
}

#[derive(Clone, Copy)]
#[repr(C, align(8))]
struct PacketHeader {
    typ: PacketType,
    conn: u32,
    size: u64,
}

impl PacketHeader {
    fn serialize_into_buffer(&self, buffer: *mut u8) {
        unsafe {
            *std::mem::transmute::<*mut u8, *mut Self>(buffer) = *self;
        }
    }
    
    fn separator() -> Self {
        Self {
            typ: PacketType::Sep,
            conn: 0,
            size: 0,
        }
    }
    
    fn data(conn: u32, size: u64) -> Self {
        Self {
            typ: PacketType::Data,
            conn,
            size,
        }
    }
    
    fn eof() -> Self {
        Self {
            typ: PacketType::Eof,
            conn: 0,
            size: 0,
        }
    }
}

const PACKET_CHANNEL_SIZE: usize = 8 * 1024 * 1024;
const PACKETS_SHM_ID: &str = "__DRAGONFLY_PACKETS_SHM_ID";

#[inline]
fn align8(x: usize) -> usize {
    let rem = x % 8;

    if rem == 0 {
        x
    } else {
        x + 8 - rem
    }
}

#[derive(Debug)]
pub struct DragonflyExecutor<OT, S, SP, I, P>
where
    OT: ObserversTuple<S>,
    S: UsesInput<Input = I>,
    SP: ShMemProvider,
    I: Input + HasPacketVector<Packet = P>,
    P: SerializeIntoBuffer,
{
    observers: OT,
    packet_channel: SP::ShMem,
    timeout: TimeSpec,
    signal: Signal,
    forkserver: Forkserver,
    phantom: PhantomData<S>,
}

impl<OT, S, SP, I, P> DragonflyExecutor<OT, S, SP, I, P>
where
    OT: ObserversTuple<S>,
    S: UsesInput<Input = I>,
    SP: ShMemProvider,
    I: Input + HasPacketVector<Packet = P>,
    P: SerializeIntoBuffer,
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
}

impl<OT, S, SP, I, P> UsesState for DragonflyExecutor<OT, S, SP, I, P>
where
    OT: ObserversTuple<S>,
    S: UsesInput<Input = I>,
    SP: ShMemProvider,
    I: Input + HasPacketVector<Packet = P>,
    P: SerializeIntoBuffer,
{
    type State = S;
}

impl<OT, S, SP, I, P> UsesObservers for DragonflyExecutor<OT, S, SP, I, P>
where
    OT: ObserversTuple<S>,
    S: UsesInput<Input = I>,
    SP: ShMemProvider,
    I: Input + HasPacketVector<Packet = P>,
    P: SerializeIntoBuffer,
{
    type Observers = OT;
}

impl<OT, S, SP, I, P> HasObservers for DragonflyExecutor<OT, S, SP, I, P>
where
    OT: ObserversTuple<S>,
    S: UsesInput<Input = I>,
    SP: ShMemProvider,
    I: Input + HasPacketVector<Packet = P>,
    P: SerializeIntoBuffer,
{
    fn observers(&self) -> &OT {
        &self.observers
    }

    fn observers_mut(&mut self) -> &mut OT {
        &mut self.observers
    }
}

impl<OT, S, SP, I, P, EM, Z> Executor<EM, Z> for DragonflyExecutor<OT, S, SP, I, P>
where
    OT: ObserversTuple<S>,
    S: UsesInput<Input = I> + std::fmt::Debug,
    SP: ShMemProvider,
    I: Input + HasPacketVector<Packet = P>,
    P: SerializeIntoBuffer + std::fmt::Debug,
    EM: UsesState<State = S>,
    Z: UsesState<State = S>,
{
    fn run_target(&mut self, _fuzzer: &mut Z, _state: &mut S, _mgr: &mut EM, input: &I) -> Result<ExitKind, Error> {
        const PACKET_HEADER_SIZE: usize = std::mem::size_of::<PacketHeader>();
        const PACKET_CHANNEL_END: usize = PACKET_CHANNEL_SIZE - PACKET_HEADER_SIZE;
        let mut exit_kind = ExitKind::Ok;
        let last_run_timed_out = self.forkserver.last_run_timed_out();

        /* Serialize input into packet channel */
        let raw_pointer = self.packet_channel.as_mut_slice().as_mut_ptr();

        PacketHeader::separator().serialize_into_buffer(raw_pointer);

        let mut last_was_sep = true;
        let mut cursor = PACKET_HEADER_SIZE;

        for packet in input.packets() {
            if cursor + PACKET_HEADER_SIZE >= PACKET_CHANNEL_END {
                break;
            }
            
            let current_pointer = unsafe { raw_pointer.add(cursor) };

            let packet_buf = unsafe {
                std::slice::from_raw_parts_mut(current_pointer.add(PACKET_HEADER_SIZE), PACKET_CHANNEL_END - cursor - PACKET_HEADER_SIZE)
            };

            if let Some(written) = packet.serialize_into_buffer(packet_buf) {
                PacketHeader::data(packet.get_connection() as u32, written as u64).serialize_into_buffer(current_pointer);
                last_was_sep = false;
                cursor += PACKET_HEADER_SIZE + align8(written);
            }

            if packet.terminates_group() && cursor + PACKET_HEADER_SIZE < PACKET_CHANNEL_END && !last_was_sep {
                PacketHeader::separator().serialize_into_buffer(unsafe { raw_pointer.add(cursor) });
                last_was_sep = true;
                cursor += PACKET_HEADER_SIZE;
            }
        }

        debug_assert!(cursor + PACKET_HEADER_SIZE <= PACKET_CHANNEL_SIZE);
        PacketHeader::eof().serialize_into_buffer(unsafe { raw_pointer.add(cursor) });

        /* Launch the client */
        let send_len = self.forkserver.write_ctl(last_run_timed_out)?;
        self.forkserver.set_last_run_timed_out(0);

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
            self.forkserver.set_last_run_timed_out(1);
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

pub struct DragonflyExecutorBuilder<'a, OT, S, SP, I, P>
where
    OT: ObserversTuple<S>,
    S: UsesInput<Input = I>,
    SP: ShMemProvider,
    I: Input + HasPacketVector<Packet = P>,
    P: SerializeIntoBuffer,
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
    phantom: PhantomData<(S, I, P)>,
}

impl<'a, OT, S, SP, I, P> DragonflyExecutorBuilder<'a, OT, S, SP, I, P>
where
    OT: ObserversTuple<S>,
    S: UsesInput<Input = I>,
    SP: ShMemProvider,
    I: Input + HasPacketVector<Packet = P>,
    P: SerializeIntoBuffer,
{
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
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

    pub fn build(self) -> Result<DragonflyExecutor<OT, S, SP, I, P>, Error> {
        macro_rules! get_value {
            ($name:ident) => {
                self.$name.ok_or(Error::illegal_argument(format!("DragonflyExecutorBuilder: {} was not set", stringify!($name))))?
            };
        }

        let shmem_provider = get_value!(shmem_provider);
        let observers = get_value!(observers);
        let timeout = get_value!(timeout);
        let program = get_value!(program);

        let mut packet_channel = shmem_provider.new_shmem(PACKET_CHANNEL_SIZE)?;
        packet_channel.write_to_env(PACKETS_SHM_ID)?;

        PacketHeader::separator().serialize_into_buffer(packet_channel.as_mut_slice().as_mut_ptr());

        let timeout = TimeSpec::milliseconds(timeout.as_millis() as i64);

        let mut forkserver = Forkserver::new(program, self.arguments, self.envs, -1, false, 0, false, self.is_deferred, self.debug_child)?;

        // Initial forkserver handshake
        let (rlen, _) = forkserver.read_st()?;

        if rlen != 4 {
            return Err(Error::unknown("Failed to start a forkserver"));
        }

        Ok(DragonflyExecutor::new(observers, packet_channel, timeout, self.signal, forkserver))
    }
}
