use std::io;

use futures::Poll;
use futures::Async;
use futures::AsyncSink;
use tokio_io::AsyncRead;
use tokio_io::AsyncWrite;
use futures::StartSend;
use futures::stream::Stream;
use futures::sink::Sink;

pub struct IntTransport<T> {
    io: T,
}

impl<T> Stream for IntTransport<T> where T: AsyncRead {
    type Item = u32;
    type Error = io::Error;

    fn poll(&mut self) -> Poll<Option<u32>, io::Error> {
        let mut buf = [0; 1];

        let n = match self.io.read(&mut buf) {
            Err(e) => {
                if e.kind() == io::ErrorKind::WouldBlock {
                    return Ok(Async::NotReady);
                } else {
                    return Err(e);
                }
            }
            Ok(0) => {
                return Ok(Async::Ready(None));
            }
            Ok(_) => {
                return Ok(Async::Ready(Some(buf[0] as u32)));
            }
        };
    }
}

impl<T> Sink for IntTransport<T> where T: AsyncWrite {
    type SinkItem = u32;
    type SinkError = io::Error;

    fn start_send(&mut self, item: u32)
        -> StartSend<u32, io::Error>
    {
        let buf = [item as u8];

        match self.io.write(&buf) {
            Err(e) => {
                if e.kind() == io::ErrorKind::WouldBlock {
                    return Ok(AsyncSink::NotReady(item));
                } else {
                    return Err(e);
                }
            }
            Ok(n) => {
                assert_eq!(1, n);
                return Ok(AsyncSink::Ready);
            }
        }
    }

    fn poll_complete(&mut self) -> Poll<(), io::Error> {
        match self.io.flush() {
            Err(e) => {
                if e.kind() == io::ErrorKind::WouldBlock {
                    return Ok(Async::NotReady);
                } else {
                    return Err(e);
                }
            }
            Ok(()) => {
                return Ok(Async::Ready(()));
            }
        }
    }
}