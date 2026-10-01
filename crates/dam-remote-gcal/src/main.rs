use std::io;

use dam_remote_gcal::serve::answer_each_request_line;

const USAGE: &str =
    "usage: dam-remote-gcal <remote> <address>\n  dam runs this; to sign in, run dam-gcal-sign-in";

fn main() {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let [remote, address] = arguments.as_slice() else {
        eprintln!("{USAGE}");
        std::process::exit(2);
    };
    answer_each_request_line(
        &mut io::stdin().lock(),
        &mut io::stdout().lock(),
        remote,
        address,
    );
}
