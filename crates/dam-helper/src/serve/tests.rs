use std::io;

use dam_protocol::{Capabilities, Mutation, PullResponse, PushResponse};

use super::serve;
use crate::Remote;

struct NeverAsked;

impl Remote for NeverAsked {
    fn capabilities(&self) -> Capabilities {
        unreachable!("no request was read")
    }
    fn pull(&self, _since: Option<&str>) -> Result<PullResponse, String> {
        unreachable!("no request was read")
    }
    fn push(&self, _mutations: Vec<Mutation>) -> Result<PushResponse, String> {
        unreachable!("no request was read")
    }
}

struct ForeverBrokenPipe;

impl io::Read for ForeverBrokenPipe {
    fn read(&mut self, _buf: &mut [u8]) -> io::Result<usize> {
        Err(io::Error::new(io::ErrorKind::BrokenPipe, "pipe broke"))
    }
}

impl io::BufRead for ForeverBrokenPipe {
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        Err(io::Error::new(io::ErrorKind::BrokenPipe, "pipe broke"))
    }
    fn consume(&mut self, _amt: usize) {}
}

#[test]
fn a_non_decode_read_error_ends_the_loop_without_spinning_forever() {
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let mut input = ForeverBrokenPipe;
        let mut out = Vec::new();
        serve(&mut input, &mut out, &NeverAsked);
        let _ = tx.send(out);
    });
    let out = rx
        .recv_timeout(std::time::Duration::from_millis(200))
        .expect("the loop to return promptly on a non-decode read error instead of spinning");
    assert!(out.is_empty());
}
