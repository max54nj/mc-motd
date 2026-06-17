use std::io::Write;
use std::net::{Ipv4Addr, Shutdown, SocketAddrV4, TcpListener, TcpStream};
use std::str::FromStr;
use std::time::Duration;

use color_eyre::eyre::{Ok, Result};

use crate::config::Config;
use crate::types::{KickPayload, ServerListPingResponse};
use crate::utils::{
    read_long, read_unsigned_short, read_utf8_string, read_varint, write_bytes_to_stream,
    write_utf8_string, write_varint, write_varint_to_stream,
};

pub struct MOTDServer {
    config: Config,
}

impl MOTDServer {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    pub fn start(self) -> Result<()> {
        color_eyre::install()?;

        let listener = TcpListener::bind(SocketAddrV4::new(
            Ipv4Addr::from_str(self.config.host.as_str()).unwrap(),
            self.config.port,
        ))
        .unwrap();

        log::info!("Running on port {}", self.config.port);

        for stream in listener.incoming() {
            Self::handle_connection(stream.unwrap(), self.config.clone());
        }

        Ok(())
    }

    fn handle_connection(mut stream: TcpStream, config: Config) {
        std::thread::spawn(move || {
            if let Err(report) = Self::handle_ping(&mut stream, config) {
                log::error!("{}", report)
            }
        });
    }

    fn handle_ping(stream: &mut TcpStream, config: Config) -> Result<()> {
        stream.set_read_timeout(Some(Duration::from_millis(200)))?;

        let mut buf: [u8; 1] = [0];
        stream.peek(&mut buf)?;

        if buf[0] == 0xFE {
            Self::handle_legacy_ping(stream, config).unwrap();
            return Ok(());
        }

        let _len = read_varint(stream)?;
        let _packet_id = read_varint(stream)?;
        let _protocol_version = read_varint(stream)?;
        let _server_address = read_utf8_string(stream)?;
        let _server_port = read_unsigned_short(stream).expect("Expected Server Port");
        let next_state = read_varint(stream)?;

        if next_state == 2 {
            let mut res_buf: Vec<u8> = Vec::new();

            let res_json = serde_json::to_string(&KickPayload {
                text: config.kick_message,
            })
            .unwrap();

            write_varint(&mut res_buf, 0);
            write_utf8_string(&mut res_buf, res_json);

            let mut status_buf: Vec<u8> = Vec::new();
            write_varint(&mut status_buf, res_buf.len() as i32);
            status_buf.append(&mut res_buf);
            write_bytes_to_stream(stream, status_buf)?;
            stream.flush()?;

            return Ok(());
        }

        let res_json = serde_json::to_string(&ServerListPingResponse {
            version: config.version,
            players: config.players,
            description: config.motd_json,
            favicon: config.favicon,
            enforces_secure_chat: false,
            previews_chat: true,
        })
        .unwrap();

        let _len = read_varint(stream)?;
        let _packet_id = read_varint(stream)?;
        let payload = read_long(stream).unwrap_or(0);

        let mut res_buf: Vec<u8> = Vec::new();
        write_varint(&mut res_buf, 0);
        write_utf8_string(&mut res_buf, res_json);

        let mut status_buf: Vec<u8> = Vec::new();
        write_varint(&mut status_buf, res_buf.len() as i32);
        status_buf.append(&mut res_buf);
        write_bytes_to_stream(stream, status_buf)?;
        stream.flush()?;

        let mut res_buf: Vec<u8> = Vec::new();
        write_varint(&mut res_buf, 1);
        res_buf.append(&mut payload.to_be_bytes().to_vec());
        write_varint_to_stream(stream, res_buf.len() as i32)?;
        stream.flush()?;
        write_bytes_to_stream(stream, res_buf)?;
        stream.flush()?;

        stream.shutdown(Shutdown::Both)?;

        Ok(())
    }

    fn handle_legacy_ping(stream: &mut TcpStream, config: Config) -> Result<()> {
        let res_string = format!(
            "$1\0{}\0{}\0{}\0{}\0{}",
            config.version.protocol,
            config.version.name,
            config.motd,
            config.players.online,
            config.players.max
        );
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

        write_bytes_to_stream(stream, res_buf)?;
        stream.flush()?;

        stream.shutdown(Shutdown::Both).unwrap();

        Ok(())
    }
}
