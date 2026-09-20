use std::io::{self, BufRead, Write};

use dam_protocol::{Request, Response, read_line, write_line};
use dam_remote_todoist::api::{ApiError, TodoistApi};
use dam_remote_todoist::{capabilities, pull, push};

fn main() {
    let stdin = io::stdin();
    let mut input = stdin.lock();
    let mut out = io::stdout().lock();
    run(&mut input, &mut out);
}

/// Reads one `Request` per line and writes the matching `Response`. A line
/// that fails to decode answers `Response::Error` and keeps going; any other
/// read failure (a broken pipe, a reset connection) ends the loop, since
/// retrying it would spin forever without ever reaching EOF.
fn run(input: &mut impl BufRead, out: &mut impl Write) {
    loop {
        let request: Option<Request> = match read_line(input) {
            Ok(r) => r,
            Err(e) if e.kind() == io::ErrorKind::InvalidData => {
                let _ = write_line(
                    out,
                    &Response::Error {
                        error: format!("cannot read the request: {e}"),
                    },
                );
                continue;
            }
            Err(_) => break,
        };
        let Some(request) = request else { break };
        let response = answer(request);
        if write_line(out, &response).is_err() {
            break;
        }
    }
}

fn answer(request: Request) -> Response {
    match request {
        Request::Capabilities => Response::Capabilities(capabilities::capabilities()),
        Request::Pull { .. } => with_api(|api| pull::pull(api).map(Response::Pull)),
        Request::Push { mutations } => {
            with_api(|api| push::push(api, mutations).map(Response::Push))
        }
    }
}

/// The one place an `ApiError` becomes a sentence. Everything above it keeps
/// the class, so a rate limit stays distinguishable from a 500 or a transport
/// failure right up to the wire.
fn with_api(f: impl FnOnce(&TodoistApi) -> Result<Response, ApiError>) -> Response {
    match TodoistApi::from_env() {
        Ok(api) => f(&api).unwrap_or_else(|e| Response::Error {
            error: e.to_string(),
        }),
        Err(error) => Response::Error { error },
    }
}

#[cfg(test)]
mod tests {
    use super::run;
    use std::io;

    /// Every read fails with a non-decode error, forever. Stands in for a
    /// broken pipe or a reset connection: `read_line` never reaches EOF.
    struct FailingReader;

    impl io::Read for FailingReader {
        fn read(&mut self, _buf: &mut [u8]) -> io::Result<usize> {
            Err(io::Error::new(io::ErrorKind::BrokenPipe, "pipe broke"))
        }
    }

    impl io::BufRead for FailingReader {
        fn fill_buf(&mut self) -> io::Result<&[u8]> {
            Err(io::Error::new(io::ErrorKind::BrokenPipe, "pipe broke"))
        }
        fn consume(&mut self, _amt: usize) {}
    }

    #[test]
    fn a_non_decode_read_error_ends_the_loop_without_spinning_forever() {
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let mut input = FailingReader;
            let mut out = Vec::new();
            run(&mut input, &mut out);
            let _ = tx.send(out);
        });
        let out = rx
            .recv_timeout(std::time::Duration::from_millis(200))
            .expect("run() to return promptly on a non-decode read error instead of spinning");
        assert!(out.is_empty());
    }
}
