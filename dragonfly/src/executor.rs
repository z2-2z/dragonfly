use libafl::prelude::{
    UsesInput, Input,
    UsesState,
    UsesObservers, ObserversTuple, HasObservers,
    Executor,
    ShMemProvider, ShMem,
    ExitKind, Error,
};
use std::marker::PhantomData;

const PACKET_CHANNEL_SIZE: usize = 8 * 1024 * 1024;
const PACKETS_SHM_ID: &str = "__DRAGONFLY_PACKETS_SHM_ID";

#[derive(Debug)]
pub struct DragonflyExecutor<OT, S, SP>
where
    OT: ObserversTuple<S>,
    S: UsesInput,
    SP: ShMemProvider,
{
    observers: OT,
    packet_channel: SP::ShMem,
    phantom: PhantomData<S>,
}

impl<OT, S, SP> DragonflyExecutor<OT, S, SP>
where
    OT: ObserversTuple<S>,
    S: UsesInput,
    SP: ShMemProvider,
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

impl<OT, S, SP> UsesState for DragonflyExecutor<OT, S, SP>
where
    OT: ObserversTuple<S>,
    S: UsesInput,
    SP: ShMemProvider,
{
    type State = S;
}

impl<OT, S, SP> UsesObservers for DragonflyExecutor<OT, S, SP>
where
    OT: ObserversTuple<S>,
    S: UsesInput,
    SP: ShMemProvider,
{
    type Observers = OT;
}

impl<OT, S, SP> HasObservers for DragonflyExecutor<OT, S, SP>
where
    OT: ObserversTuple<S>,
    S: UsesInput,
    SP: ShMemProvider,
{
    fn observers(&self) -> &OT {
        &self.observers
    }

    fn observers_mut(&mut self) -> &mut OT {
        &mut self.observers
    }
}

impl<OT, S, SP, EM, Z> Executor<EM, Z> for DragonflyExecutor<OT, S, SP>
where
    OT: ObserversTuple<S>,
    S: UsesInput + std::fmt::Debug,
    SP: ShMemProvider,
    EM: UsesState<State = S>,
    Z: UsesState<State = S>,
{
    fn run_target(
        &mut self,
        fuzzer: &mut Z,
        state: &mut S,
        mgr: &mut EM,
        input: &Self::Input,
    ) -> Result<ExitKind, Error> {
        Ok(ExitKind::Ok)
    }
}
