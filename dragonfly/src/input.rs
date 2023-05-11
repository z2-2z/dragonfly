
pub trait SerializeIntoShMem {
    fn serialize_into_shm(&self, shmem: &mut [u8]) -> Option<usize>;
}

pub trait HasPacketVector {
    type Packet: SerializeIntoShMem;
    
    fn packets(&self) -> &[Self::Packet];
    fn packets_mut(&mut self) -> &mut Vec<Self::Packet>;
}


//TODO: impl SerializeIntoShMem for Inputs from libafl
