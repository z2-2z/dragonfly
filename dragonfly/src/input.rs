use libafl::prelude::{
    BytesInput,
    HasBytesVec,
    HasLen,
};

pub trait SerializeIntoShMem {
    fn serialize_into_shm(&self, shmem: &mut [u8]) -> Option<usize>;
}

impl SerializeIntoShMem for BytesInput {
    fn serialize_into_shm(&self, shmem: &mut [u8]) -> Option<usize> {
        let len = std::cmp::min(shmem.len(), self.len());
        shmem[..len].copy_from_slice(&self.bytes()[..len]);
        Some(len)
    }
}

pub trait HasPacketVector {
    type Packet: SerializeIntoShMem + Clone;

    fn packets(&self) -> &[Self::Packet];
    fn packets_mut(&mut self) -> &mut Vec<Self::Packet>;
}

//TODO: impl SerializeIntoShMem for Inputs from libafl
