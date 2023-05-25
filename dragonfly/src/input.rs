use libafl::prelude::{
    BytesInput,
    HasBytesVec,
    HasLen,
};

pub trait SerializeIntoBuffer {
    fn serialize_into_buffer(&self, buffer: &mut [u8]) -> Option<usize>;
    fn get_connection(&self) -> usize;

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

    fn get_connection(&self) -> usize {
        0
    }
}

pub trait HasPacketVector {
    type Packet: SerializeIntoBuffer + Clone;

    fn packets(&self) -> &[Self::Packet];
    fn packets_mut(&mut self) -> &mut Vec<Self::Packet>;
}
