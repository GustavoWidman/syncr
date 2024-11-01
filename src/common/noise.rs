use snow::{params::NoiseParams, Builder, Error, TransportState};

use std::{
    io::{self, Read, Write},
    net::TcpStream,
};

pub struct NoiseEncoder {
    PARAMS: NoiseParams,
    SECRET: Vec<u8>,
}

impl NoiseEncoder {
    pub fn new(secret: &[u8], params: &str) -> Self {
        Self {
            PARAMS: params.parse().unwrap(),
            SECRET: secret.to_vec(),
        }
    }

    fn recv(stream: &mut TcpStream) -> io::Result<Vec<u8>> {
        let mut msg_len_buf = [0u8; 2];

        stream.read_exact(&mut msg_len_buf)?;

        let msg_len = usize::from(u16::from_be_bytes(msg_len_buf));
        let mut msg = vec![0u8; msg_len];

        stream.read_exact(&mut msg[..])?;

        Ok(msg)
    }

    fn send(stream: &mut TcpStream, buf: &[u8]) {
        let len = u16::try_from(buf.len()).expect("message too large");
        stream.write_all(&len.to_be_bytes()).unwrap();
        stream.write_all(buf).unwrap();
    }

    pub fn handshake_server_side(
        &mut self,
        stream: &mut TcpStream,
    ) -> Result<TransportState, Error> {
        let builder = Builder::new(self.PARAMS.clone());
        let static_key = builder.generate_keypair().unwrap().private;
        let mut noise = builder
            .local_private_key(&static_key)
            .psk(3, self.SECRET.as_slice())
            .build_responder()
            .unwrap();

        let mut buf = vec![0u8; 65535];

        // <- e
        noise
            .read_message(&NoiseEncoder::recv(stream).unwrap(), &mut buf)
            .unwrap();

        // -> e, ee, s, es
        let len = noise.write_message(&[], &mut buf).unwrap();
        NoiseEncoder::send(stream, &buf[..len]);

        // <- s, se
        noise
            .read_message(&NoiseEncoder::recv(stream).unwrap(), &mut buf)
            .unwrap();

        return noise.into_transport_mode();
    }

    pub fn handhsake_client_side(
        &mut self,
        stream: &mut TcpStream,
    ) -> Result<TransportState, Error> {
        let builder = Builder::new(self.PARAMS.clone());
        let static_key = builder.generate_keypair().unwrap().private;
        let mut noise = builder
            .local_private_key(&static_key)
            .psk(3, self.SECRET.as_slice())
            .build_initiator()
            .unwrap();

        let mut buf = vec![0u8; 65535];

        // -> e
        let len = noise.write_message(&[], &mut buf).unwrap();
        NoiseEncoder::send(stream, &buf[..len]);

        // <- e, ee, s, es
        noise
            .read_message(&NoiseEncoder::recv(stream).unwrap(), &mut buf)
            .unwrap();

        // -> s, se
        let len = noise.write_message(&[], &mut buf).unwrap();
        NoiseEncoder::send(stream, &buf[..len]);

        return noise.into_transport_mode();
    }
}

pub struct Transport {
    stream: TcpStream,
    noise: TransportState,
}

impl Transport {
    pub fn new(stream: TcpStream, noise: TransportState) -> Self {
        Self { stream, noise }
    }

    pub fn recv(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let mut msg_len_buf = [0u8; 2];
        self.stream.read(&mut msg_len_buf)?;
        let msg_len = usize::from(u16::from_be_bytes(msg_len_buf));

        match msg_len {
            0 => Ok(0),
            _ => {
                let mut msg = vec![0u8; msg_len];

                self.stream.read_exact(&mut msg[..])?;

                match self.noise.read_message(&msg, buf) {
                    Ok(n) => Ok(n),
                    Err(e) => Err(io::Error::new(io::ErrorKind::Other, e)),
                }
            }
        }
    }

    pub fn send(&mut self, msg: &[u8]) -> io::Result<()> {
        let mut buf = [0u8; 65535];

        let len = self.noise.write_message(msg, &mut buf);

        match len {
            Ok(len) => {
                let len = u16::try_from(buf[..len].len()).expect("message too large");
                print!("len: {:?}", len);
                self.stream.write(&len.to_be_bytes())?;
                self.stream.write(&mut buf[..len.into()])?;

                Ok(())
            }
            Err(e) => Err(io::Error::new(io::ErrorKind::Other, e)),
        }
    }
}

macro_rules! handshake_server {
    ($secret:expr, $params:expr, $stream:expr) => {
        crate::common::noise::NoiseEncoder::new($secret, $params).handshake_server_side($stream)
    };
}

macro_rules! handshake_client {
    ($secret:expr, $params:expr, $stream:expr) => {
        crate::common::noise::NoiseEncoder::new($secret, $params).handhsake_client_side($stream)
    };
}

pub(crate) use handshake_client;
pub(crate) use handshake_server;
