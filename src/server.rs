use std::io::Read;
use std::net::{Ipv4Addr, Shutdown, SocketAddrV4, TcpListener, TcpStream};
use std::str::FromStr;

use color_eyre::Report;
use color_eyre::eyre::{Ok, Result, eyre};

use crate::types::{Description, Players, ServerListPingResponse, Version};
use crate::utils::{
    read_long, read_unsigned_short, read_utf8_string, write_bytes_to_stream, write_utf8_string,
    write_varint, write_varint_to_stream,
};

pub struct MOTDServer {
    port: u16,
}

impl MOTDServer {
    pub fn new(port: u16) -> Self {
        Self { port }
    }

    pub fn start(self) -> Result<()> {
        color_eyre::install();

        let listener = TcpListener::bind(SocketAddrV4::new(
            Ipv4Addr::from_str("127.0.0.1").unwrap(),
            self.port,
        ))
        .unwrap();

        println!("Running on port {}", self.port);

        for stream in listener.incoming() {
            Self::handle_connection(stream.unwrap());
        }

        Ok(())
    }

    fn handle_connection(mut stream: TcpStream) {
        std::thread::spawn(move || {
            if let Err(report) = Self::handle_ping(&mut stream) {
                eprintln!("{}", report)
            }
        });
    }

    fn handle_ping(stream: &mut TcpStream) -> Result<()> {
        println!("--- New connection ---");

        let mut buf: [u8; 1] = [0];
        stream.peek(&mut buf);

        if buf[0] == 0xFE {
            println!("Legacy server list ping packet");
            Self::handle_legacy_ping(stream).unwrap();
            return Ok(());
        }

        let _len = read_varint(stream);
        let _packet_id = read_varint(stream);
        let _protocol_version = read_varint(stream);
        let _server_address = read_utf8_string(stream);
        let _server_port = read_unsigned_short(stream);
        let next_state = read_varint(stream);

        if next_state != 1 {
            stream
                .shutdown(Shutdown::Both)
                .expect("Failed to shutdown stream");
            return Err(eyre!("Client tried to join"));
        }

        let _len = read_varint(stream);
        let _packet_id = read_varint(stream);

        let res_json = serde_json::to_string(&ServerListPingResponse {
            version: Version {
                name: "1.21.11".to_string(),
                protocol: 774,
            },
            players: Players {
                max: 10,
                online: 2,
                sample: vec![],
            },
            description: Description {
                text: "MOTD".to_string(),
            },
            favicon: None,
            enforces_secure_chat: false,
            previews_chat: true,
        })
        .unwrap();

        let mut res_buf: Vec<u8> = Vec::new();
        write_varint(&mut res_buf, 0);
        write_utf8_string(&mut res_buf, res_json);

        write_varint_to_stream(stream, res_buf.len() as i32);
        write_bytes_to_stream(stream, res_buf);

        let mut len = [0];
        match stream.read(&mut len).ok() {
            Some(n) => {
                if n == 0 {
                    return Ok(());
                }
            }
            None => {
                return Err(eyre!("Failed to read from stream"));
            }
        };
        let _packet_id = read_varint(stream);
        let payload = read_long(stream);

        let mut res_buf: Vec<u8> = Vec::new();
        write_varint(&mut res_buf, 1);
        res_buf.append(&mut payload.to_be_bytes().to_vec());
        write_varint_to_stream(stream, res_buf.len() as i32);
        write_bytes_to_stream(stream, res_buf);

        stream.shutdown(Shutdown::Both)?;

        Ok(())
    }

    fn handle_legacy_ping(stream: &mut TcpStream) -> Result<()> {
        let res_string = format!("$1\0{}\0{}\0{}\0{}\0{}", "774", "1.21.11", "MOTD", 2, 10);
        // let res_string = format!("$1\0{}\0{}\0{}\0{}\0{}", "78", "1.6", "MOTD", 2, 10);
        let mut res_buf: Vec<u8> = Vec::new();

        res_buf.push(0xFF);

        let string_len = (res_string.len() as u16 - 1).to_be_bytes();
        res_buf.push(string_len[0]);
        res_buf.push(string_len[1]);

        let data: Vec<u16> = res_string
            .encode_utf16()
            .collect::<Vec<u16>>()
            .iter()
            .map(|n| u16::from_be_bytes([(n & 0xFF) as u8, (n >> 8) as u8]))
            .collect();

        unsafe {
            res_buf.append(&mut data.align_to::<u8>().1.to_vec());
        }

        write_bytes_to_stream(stream, res_buf);

        stream.shutdown(Shutdown::Both).unwrap();

        Ok(())
    }
}

fn read_string(stream: &mut impl Read) -> String {
    let len = read_varint(stream) as usize;
    let mut buf = vec![0u8; len];
    stream.read_exact(&mut buf).unwrap();
    String::from_utf8(buf).unwrap()
}

fn read_varint(stream: &mut impl Read) -> i32 {
    let mut num = 0i32;
    let mut shift = 0;

    loop {
        let mut buf = [0u8; 1];
        stream.read_exact(&mut buf).unwrap();
        let byte = buf[0];

        num |= ((byte & 0x7F) as i32) << shift;

        if byte & 0x80 == 0 {
            break;
        }

        shift += 7;

        if shift > 35 {
            panic!("VarInt too big");
        }
    }

    num
}
