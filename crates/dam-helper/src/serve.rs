use std::io::{self, BufRead, Write};

use dam_protocol::{Request, Response, read_line, write_line};

use crate::Remote;

pub fn serve(input: &mut impl BufRead, out: &mut impl Write, remote: &impl Remote) {
    loop {
        let response = match read_line::<Request>(input) {
            Ok(Some(request)) => answer(remote, request),
            Err(undecodable) if undecodable.kind() == io::ErrorKind::InvalidData => {
                Response::Error {
                    error: format!("cannot read the request: {undecodable}"),
                }
            }
            Ok(None) | Err(_) => break,
        };
        if write_line(out, &response).is_err() {
            break;
        }
    }
}

fn answer(remote: &impl Remote, request: Request) -> Response {
    let answered = match request {
        Request::Capabilities => Ok(Response::Capabilities(remote.capabilities())),
        Request::Pull { since } => remote.pull(since.as_deref()).map(Response::Pull),
        Request::Push { mutations } => remote.push(mutations).map(Response::Push),
    };
    answered.unwrap_or_else(|error| Response::Error { error })
}

#[cfg(test)]
mod tests;
