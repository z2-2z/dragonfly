use ahash::AHasher;
use libafl::prelude::{
    BytesInput,
    HasBytesVec,
    HasLen,
    Input,
};
use serde::{
    Deserialize,
    Serialize,
};
use std::hash::{
    Hash,
    Hasher,
};

pub trait SerializeIntoBuffer {
    fn serialize_into_buffer(&self, buffer: &mut [u8]) -> Option<usize>;

    fn get_connection(&self) -> usize {
        0
    }

    fn terminates_group(&self) -> bool {
        true
    }
}

impl SerializeIntoBuffer for BytesInput {
    fn serialize_into_buffer(&self, buffer: &mut [u8]) -> Option<usize> {
        let len = std::cmp::min(buffer.len(), self.len());
        buffer[..len].copy_from_slice(&self.bytes()[..len]);
        Some(len)
    }
}

pub trait HasPacketVector {
    type Packet: SerializeIntoBuffer + Clone;

    fn packets(&self) -> &[Self::Packet];
    fn packets_mut(&mut self) -> &mut Vec<Self::Packet>;
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(bound = "P: Serialize + for<'a> Deserialize<'a>")]
pub struct DragonflyInput<P> {
    packets: Vec<P>,
}

impl<P> DragonflyInput<P> {
    pub fn new(packets: Vec<P>) -> Self {
        Self {
            packets,
        }
    }
}

impl<P> HasPacketVector for DragonflyInput<P>
where
    P: SerializeIntoBuffer + Clone,
{
    type Packet = P;

    fn packets(&self) -> &[Self::Packet] {
        &self.packets
    }

    fn packets_mut(&mut self) -> &mut Vec<Self::Packet> {
        &mut self.packets
    }
}

impl<P> Input for DragonflyInput<P>
where
    P: std::fmt::Debug + Serialize + for<'a> Deserialize<'a> + Clone + Hash,
{
    fn generate_name(&self, _idx: usize) -> String {
        let mut hasher = AHasher::default();
        hasher.write_usize(self.packets.len());

        for packet in &self.packets {
            packet.hash(&mut hasher);
        }

        let digest = hasher.finish();
        format!("dragonfly-{:016x}", digest)
    }
}

impl<P> HasLen for DragonflyInput<P> {
    fn len(&self) -> usize {
        self.packets.len()
    }
}
