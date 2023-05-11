use libafl::prelude::{
    UsesInput, Input,
    UsesState,
    UsesObservers, ObserversTuple, HasObservers,
    Executor, AsMutSlice,
    ShMemProvider, ShMem,
    ExitKind, Error,
};
use std::marker::PhantomData;

use crate::input::{
    HasPacketVector,
    SerializeIntoShMem,
};

const PACKET_CHANNEL_SIZE: usize = 8 * 1024 * 1024;
const PACKETS_SHM_ID: &str = "__DRAGONFLY_PACKETS_SHM_ID";

#[derive(Debug)]
pub struct DragonflyExecutor<OT, S, SP, I, P>
where
    OT: ObserversTuple<S>,
    S: UsesInput<Input = I>,
    SP: ShMemProvider,
    I: Input + HasPacketVector<Packet = P>,
    P: SerializeIntoShMem,
{
    observers: OT,
    packet_channel: SP::ShMem,
    phantom: PhantomData<S>,
}

impl<OT, S, SP, I, P> DragonflyExecutor<OT, S, SP, I, P>
where
    OT: ObserversTuple<S>,
    S: UsesInput<Input = I>,
    SP: ShMemProvider,
    I: Input + HasPacketVector<Packet = P>,
    P: SerializeIntoShMem,
{
    pub fn new(observers: OT, shmem_provider: &mut SP) -> Result<Self, Error> {
        let packet_channel = shmem_provider.new_shmem(PACKET_CHANNEL_SIZE)?;
        packet_channel.write_to_env(PACKETS_SHM_ID)?;
        
        Ok(Self {
            observers,
            packet_channel,
            phantom: PhantomData,
        })
    }
}

impl<OT, S, SP, I, P> UsesState for DragonflyExecutor<OT, S, SP, I, P>
where
    OT: ObserversTuple<S>,
    S: UsesInput<Input = I>,
    SP: ShMemProvider,
    I: Input + HasPacketVector<Packet = P>,
    P: SerializeIntoShMem,
{
    type State = S;
}

impl<OT, S, SP, I, P> UsesObservers for DragonflyExecutor<OT, S, SP, I, P>
where
    OT: ObserversTuple<S>,
    S: UsesInput<Input = I>,
    SP: ShMemProvider,
    I: Input + HasPacketVector<Packet = P>,
    P: SerializeIntoShMem,
{
    type Observers = OT;
}

impl<OT, S, SP, I, P> HasObservers for DragonflyExecutor<OT, S, SP, I, P>
where
    OT: ObserversTuple<S>,
    S: UsesInput<Input = I>,
    SP: ShMemProvider,
    I: Input + HasPacketVector<Packet = P>,
    P: SerializeIntoShMem,
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
    P: SerializeIntoShMem + std::fmt::Debug,
    EM: UsesState<State = S>,
    Z: UsesState<State = S>,
{
    fn run_target(&mut self, _fuzzer: &mut Z, _state: &mut S, _mgr: &mut EM, input: &I) -> Result<ExitKind, Error> {
        for packet in input.packets() {
            packet.serialize_into_shm(self.packet_channel.as_mut_slice());
        }
        
        Ok(ExitKind::Ok)
    }
}
