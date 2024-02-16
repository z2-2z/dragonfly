use clap::Parser;
use dragonfly::components::DragonflyInput;
use std::io::Write;

#[derive(Debug)]
struct Packet {
    connection: usize,
    terminates_group: bool,
    data: Vec<u8>,
}

impl Packet {
    fn new(desc: String) -> Self {
        let desc = desc.as_bytes();
        let mut i = 0;
        let mut terminates_group = false;
        
        while i < desc.len() {
            if desc[i] == b':' {
                break;
            }
            
            i += 1;
        }
        
        assert!(i < desc.len());
        
        let connection = std::str::from_utf8(&desc[..i]).unwrap();
        let connection = connection.parse::<usize>().unwrap();
        
        i += 1;
        
        while i < desc.len() {
            match desc[i] {
                b's' | b'S' => terminates_group = true,
                b':' => break,
                _ => panic!("invalid flag char: {}", desc[i]),
            }
            
            i += 1;
        }
        
        assert!(i < desc.len());
        i +=  1;
        
        let data = desc[i..].to_vec();
        
        Packet {
            connection,
            terminates_group,
            data,
        }
    }
}

impl dragonfly::components::Packet for Packet {
    fn serialize_content(&self, buffer: &mut [u8]) -> Option<usize> {
        if self.data.is_empty() {
            return None;
        }
        
        let len = std::cmp::min(buffer.len(), self.data.len());
        buffer[..len].copy_from_slice(&self.data[..len]);
        Some(len)
    }
    
    fn terminates_group(&self) -> bool {
        self.terminates_group
    }
    
    fn connection(&self) -> usize {
        self.connection
    }
}

#[derive(clap::Parser)]
#[command(version, about, long_about = None)]
struct Args {
    description: Vec<String>,
}

#[allow(clippy::result_unit_err)]
pub fn main() -> Result<(), ()> {
    let args = Args::parse();
    let mut packets = Vec::new();
    
    for packet_desc in args.description {
        let packet = Packet::new(packet_desc);
        packets.push(packet);
    }
    
    let input = DragonflyInput::new(packets);
    let mut buf = vec![0; 8 * 1024 * 1024];
    
    let len = input.serialize_dragonfly_format(&mut buf);
    
    if len == buf.len() {
        return Err(());
    }
    
    std::io::stdout().write_all(&buf[..len]).unwrap();
    
    Ok(())
}
