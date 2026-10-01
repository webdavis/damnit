use std::io::{self, BufRead, Write};

use dam_protocol::{Request, Response, read_line, write_line};

use crate::api::{ApiError, TodoistApi};
use crate::{capabilities, pull, push};

pub fn answer_each_request_line(input: &mut impl BufRead, out: &mut impl Write, remote: &str) {
    loop {
        let request = match read_line::<Request>(input) {
            Ok(Some(request)) => request,
            Err(undecodable) if undecodable.kind() == io::ErrorKind::InvalidData => {
                let _ = write_line(out, &cannot_read(&undecodable));
                continue;
            }
            Ok(None) | Err(_) => break,
        };
        let response = answer(request, remote);
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

fn answer(request: Request, remote: &str) -> Response {
    match request {
        Request::Capabilities => Response::Capabilities(capabilities::capabilities()),
        Request::Pull { .. } => {
            with_api_error_as_sentence(remote, |api| pull::pull(api).map(Response::Pull))
        }
        Request::Push { mutations } => {
            with_api_error_as_sentence(remote, |api| push::push(api, mutations).map(Response::Push))
        }
    }
}

fn with_api_error_as_sentence(
    remote: &str,
    f: impl FnOnce(&TodoistApi) -> Result<Response, ApiError>,
) -> Response {
    match TodoistApi::from_env(remote) {
        Ok(api) => f(&api).unwrap_or_else(|e| Response::Error {
            error: e.to_string(),
        }),
        Err(error) => Response::Error { error },
    }
}

#[cfg(test)]
mod tests;
