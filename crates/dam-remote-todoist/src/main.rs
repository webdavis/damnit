use std::io;

use dam_protocol::{Request, Response, read_line, write_line};
use dam_remote_todoist::api::TodoistApi;
use dam_remote_todoist::{capabilities, pull, push};

fn main() {
    let stdin = io::stdin();
    let mut input = stdin.lock();
    let mut out = io::stdout().lock();
    loop {
        let request: Option<Request> = match read_line(&mut input) {
            Ok(r) => r,
            Err(e) => {
                let _ = write_line(
                    &mut out,
                    &Response::Error {
                        error: format!("cannot read the request: {e}"),
                    },
                );
                continue;
            }
        };
        let Some(request) = request else { break };
        let response = answer(request);
        if write_line(&mut out, &response).is_err() {
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

fn with_api(f: impl FnOnce(&TodoistApi) -> Result<Response, String>) -> Response {
    match TodoistApi::from_env() {
        Ok(api) => f(&api).unwrap_or_else(|error| Response::Error { error }),
        Err(error) => Response::Error { error },
    }
}
