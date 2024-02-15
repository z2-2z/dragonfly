use ahash::AHasher;
use libafl_bolts::HasLen;
use libafl::prelude::Input;
use serde::{
    Deserialize,
    Serialize,
};
use std::hash::{
    Hash,
    Hasher,
};

pub trait Packet {
    fn serialize_content(&self, buffer: &mut [u8]) -> Option<usize>;

    fn connection(&self) -> usize {
        0
    }

    fn terminates_group(&self) -> bool {
        true
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(bound = "P: Serialize + for<'a> Deserialize<'a>")]
pub struct DragonflyInput<P>
where
    P: Packet,
{
    packets: Vec<P>,
}

impl<P> DragonflyInput<P>
where
    P: Packet,
{
    pub fn new(packets: Vec<P>) -> Self {
        Self {
            packets,
        }
    }
    
    pub fn packets(&self) -> &[P] {
        &self.packets
    }

    pub fn packets_mut(&mut self) -> &mut Vec<P> {
        &mut self.packets
    }
}

impl<P> Input for DragonflyInput<P>
where
    P: Packet + std::fmt::Debug + Serialize + for<'a> Deserialize<'a> + Clone + Hash,
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

impl<P> HasLen for DragonflyInput<P>
where
    P: Packet,
{
    fn len(&self) -> usize {
        self.packets.len()
    }
}

/*** Dragonfly Serialization Stuff ***/

#[repr(u32)]
enum PacketType {
    Data = 1,
    Sep = 2,
    Eof = 3,
}

#[repr(C, align(8))]
struct PacketHeader {
    typ: PacketType,
    conn: u32,
    size: u64,
}

impl PacketHeader {
    const SIZE: usize = std::mem::size_of::<PacketHeader>();
    
    #[inline]
    fn separator() -> Self {
        Self {
            typ: PacketType::Sep,
            conn: 0,
            size: 0,
        }
    }
    
    #[inline]
    fn data(conn: u32, size: u64) -> Self {
        Self {
            typ: PacketType::Data,
            conn,
            size,
        }
    }
    
    #[inline]
    fn eof() -> Self {
        Self {
            typ: PacketType::Eof,
            conn: 0,
            size: 0,
        }
    }
}

fn align8(x: usize) -> usize {
    let rem = x % 8;

    if rem == 0 {
        x
    } else {
        x + 8 - rem
    }
}

impl<P> DragonflyInput<P>
where
    P: Packet,
{
    pub fn serialize_dragonfly_format(&self, buffer: &mut [u8]) -> usize {
        let end = buffer.len().saturating_sub(PacketHeader::SIZE);
        debug_assert!(end >= PacketHeader::SIZE);
        let mut cursor = 0;
        let mut last_was_sep = true;
        
        /* First, put a separator */
        unsafe {
            *std::mem::transmute::<*mut u8, *mut PacketHeader>(buffer.as_mut_ptr()) = PacketHeader::separator();
        }
        cursor += PacketHeader::SIZE;
        
        /* Then, serialize all packets */
        for packet in self.packets() {
            let start_header = cursor;
            cursor += PacketHeader::SIZE;
            
            if cursor >= end {
                break;
            }
            
            /* If packet contains data, write data */
            if let Some(packet_size) = packet.serialize_content(&mut buffer[cursor..end]) {
                let header = PacketHeader::data(packet.connection() as u32, packet_size as u64);
                unsafe {
                    *std::mem::transmute::<*mut u8, *mut PacketHeader>(buffer[start_header..].as_mut_ptr()) = header;
                }
                cursor = std::cmp::min(cursor + align8(packet_size), end);
                last_was_sep = false;
            }
            
            /* If packet terminates a group, place a group separator */
            if packet.terminates_group() && cursor + PacketHeader::SIZE < end && !last_was_sep {
                unsafe {
                    *std::mem::transmute::<*mut u8, *mut PacketHeader>(buffer[cursor..].as_mut_ptr()) = PacketHeader::separator();
                }
                cursor += PacketHeader::SIZE;
                last_was_sep = true;
            }
        }
        
        /* Finally, place an eof marker */
        debug_assert!(cursor <= end);
        unsafe {
            *std::mem::transmute::<*mut u8, *mut PacketHeader>(buffer[cursor..].as_mut_ptr()) = PacketHeader::eof();
        }
        
        cursor + PacketHeader::SIZE
    }
}
