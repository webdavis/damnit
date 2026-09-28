use std::io::{self, BufRead, Write};

use dam_protocol::{Request, Response, read_line, write_line};
use dam_remote_todoist::api::{ApiError, TodoistApi};
use dam_remote_todoist::{capabilities, pull, push};

fn main() {
    let stdin = io::stdin();
    let mut input = stdin.lock();
    let mut out = io::stdout().lock();
    answer_each_request_line(&mut input, &mut out);
}

fn answer_each_request_line(input: &mut impl BufRead, out: &mut impl Write) {
    loop {
        let request = match read_line::<Request>(input) {
            Ok(Some(request)) => request,
            Err(undecodable) if undecodable.kind() == io::ErrorKind::InvalidData => {
                let _ = write_line(out, &cannot_read(&undecodable));
                continue;
            }
            Ok(None) | Err(_) => break,
        };
        let response = answer(request);
        if write_line(out, &response).is_err() {
            break;
        }
    }
}

fn cannot_read(undecodable: &io::Error) -> Response {
    Response::Error {
        error: format!("cannot read the request: {undecodable}"),
    }
}

fn answer(request: Request) -> Response {
    match request {
        Request::Capabilities => Response::Capabilities(capabilities::capabilities()),
        Request::Pull { .. } => {
            with_api_error_as_sentence(|api| pull::pull(api).map(Response::Pull))
        }
        Request::Push { mutations } => {
            with_api_error_as_sentence(|api| push::push(api, mutations).map(Response::Push))
        }
    }
}

fn with_api_error_as_sentence(
    f: impl FnOnce(&TodoistApi) -> Result<Response, ApiError>,
) -> Response {
    match TodoistApi::from_env() {
        Ok(api) => f(&api).unwrap_or_else(|e| Response::Error {
            error: e.to_string(),
        }),
        Err(error) => Response::Error { error },
    }
}

#[cfg(test)]
mod tests {
    use super::answer_each_request_line;
    use std::io;

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
            answer_each_request_line(&mut input, &mut out);
            let _ = tx.send(out);
        });
        let out = rx
            .recv_timeout(std::time::Duration::from_millis(200))
            .expect("the loop to return promptly on a non-decode read error instead of spinning");
        assert!(out.is_empty());
    }
}
