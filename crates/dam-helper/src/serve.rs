use std::io::{self, BufRead, Write};

use dam_protocol::{Reply, Request, read_line, write_line};

use crate::Remote;

pub fn serve(input: &mut impl BufRead, out: &mut impl Write, remote: &impl Remote) {
    loop {
        let written = match read_line::<Request>(input) {
            Ok(Some(request)) => answer(out, remote, request),
            Err(undecodable) if undecodable.kind() == io::ErrorKind::InvalidData => write_line(
                out,
                &Reply::<()>::Failed {
                    error: format!("cannot read the request: {undecodable}"),
                },
            ),
            Ok(None) | Err(_) => break,
        };
        if written.is_err() {
            break;
        }
    }
}

fn answer(out: &mut impl Write, remote: &impl Remote, request: Request) -> io::Result<()> {
    match request {
        Request::Capabilities => write_line(out, &Reply::Answered(remote.capabilities())),
        Request::Pull { since } => write_line(out, &Reply::from(remote.pull(since.as_deref()))),
        Request::Push { mutations } => write_line(out, &Reply::from(remote.push(mutations))),
    }
}

#[cfg(test)]
mod tests;
