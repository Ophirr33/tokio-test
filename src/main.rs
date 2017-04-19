extern crate bytes;
extern crate futures;
extern crate tokio_core;
extern crate tokio_io;
extern crate tokio_proto;
extern crate tokio_service;

use std::str;
use std::io::{self, Error, ErrorKind, Write, Read};
use std::net::TcpStream;
use std::thread;
use std::time::Duration;

use bytes::{BigEndian, Buf, BufMut, ByteOrder, BytesMut, IntoBuf};
use futures::{future, Future, BoxFuture};
use tokio_io::codec::{Decoder, Encoder, Framed};
use tokio_io::{AsyncRead, AsyncWrite};
use tokio_proto::TcpServer;
use tokio_proto::pipeline::{ServerProto};
use tokio_service::Service;

// First, we implement a *codec*, which provides a way of encoding and
// decoding messages for the protocol. See the documentation for `Framed`,
// `Decoder`, and `Encoder in `tokio-io` for more details on how that works.

#[derive(Default)]
pub struct IntCodec;

impl Decoder for IntCodec {
    type Item = u64;
    type Error = Error;

    // Attempts to decode a message from the given buffer if a complete
    // message is available; returns `Ok(None)` if the buffer does not
    // yet hold a complete message.
    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<u64>, Error> {
        if src.len() < 8 {
            return Ok(None);
        }
        let eight_bytes = src.split_to(8);
        let num = eight_bytes.into_buf().get_u64::<BigEndian>();
        Ok(Some(num))
    }
}

impl Encoder for IntCodec {
    type Item = u64;
    type Error = Error;

    // Write the u64 into the destination buffer
    fn encode(&mut self, item: u64, dst: &mut BytesMut) -> Result<(), Error> {
        let cap = dst.remaining_mut();
        if cap < 8 {
            // Not enough room to write the header + u64
            Err(Error::new(ErrorKind::WriteZero,
                           format!("Not enough room in dst bytes, requires 8 open bytes, found \
                                    {}",
                                   cap)))
        } else {
            Ok(dst.put_u64::<BigEndian>(item))
        }
    }
}

// Next, we implement the server protocol, which just hooks up the codec above.

pub struct IntProto;

impl<T: AsyncRead + AsyncWrite + 'static> ServerProto<T> for IntProto {
    type Request = u64;
    type Response = u64;
    type Transport = Framed<T, IntCodec>;
    type BindTransport = Result<Self::Transport, io::Error>;

    fn bind_transport(&self, io: T) -> Self::BindTransport {
        Ok(io.framed(IntCodec))
    }
}

// Now we implement a service we'd like to run on top of this protocol

pub struct Doubler;

impl Service for Doubler {
    type Request = u64;
    type Response = u64;
    type Error = io::Error;
    type Future = BoxFuture<u64, io::Error>;

    fn call(&self, req: u64) -> Self::Future {
        // Just return the request, doubled
        future::finished(req * 2).boxed()
    }
}

// Finally, we can actually host this service locally!
fn main() {
    let addr = "127.0.0.1:12345".parse().unwrap();
    thread::spawn(move || {
        TcpServer::new(IntProto, addr).serve(|| Ok(Doubler));
    });

    thread::sleep(Duration::new(1, 0));
    let mut stream = TcpStream::connect(addr).expect("Could not connect to addr");
    let mut buff = [0; 8];
    for i in 0..100000 {
        BigEndian::write_u64(&mut buff, i);
        let _ = stream.write(&buff).expect(&format!("Failed to write {}", i));
        let read_in = stream.read(&mut buff).expect(&format!("Failed to read {}", i));
        if read_in != 8 {
            println!("Only read {} bytes", read_in);
            continue
        }
        let result = BigEndian::read_u64(&buff);
        assert_eq!(i*2, result)
    }
}
