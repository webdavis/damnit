use std::io::{self, BufRead, Write};

use dam_protocol::{PullResponse, Request, Response, read_line, write_line};

use crate::access::access_token;
use crate::address::calendars;
use crate::calendar_api::CalendarApi;
use crate::http::agent;
use crate::{Credentials, Endpoints, capabilities, pull, push};

pub fn answer_each_request_line(
    input: &mut impl BufRead,
    out: &mut impl Write,
    remote: &str,
    address: &str,
) {
    loop {
        let request = match read_line::<Request>(input) {
            Ok(Some(request)) => request,
            Err(undecodable) if undecodable.kind() == io::ErrorKind::InvalidData => {
                let _ = write_line(
                    out,
                    &Response::Error {
                        error: format!("cannot read the request: {undecodable}"),
                    },
                );
                continue;
            }
            Ok(None) | Err(_) => break,
        };
        if write_line(out, &answer(request, remote, address)).is_err() {
            break;
        }
    }
}

fn answer(request: Request, remote: &str, address: &str) -> Response {
    match request {
        Request::Capabilities => Response::Capabilities(capabilities::capabilities()),
        Request::Push { mutations } => Response::Push(push::refuse(&mutations)),
        Request::Pull { since } => match pull_now(remote, address, since.as_deref()) {
            Ok(pulled) => Response::Pull(pulled),
            Err(error) => Response::Error { error },
        },
    }
}

fn pull_now(remote: &str, address: &str, since: Option<&str>) -> Result<PullResponse, String> {
    let credentials = Credentials::from_env(remote)?;
    let endpoints = Endpoints::from_env()?;
    let agent = agent();
    let access = access_token(&agent, &endpoints, &credentials).map_err(|e| e.to_string())?;
    let api = CalendarApi::new(agent, &endpoints, access);
    pull::pull(&api, &calendars(address), since, jiff::Timestamp::now()).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests;
