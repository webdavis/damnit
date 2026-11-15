use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use dam_application::FunctionFault;

use super::filter_support::{every_field_task, run_filter_limited};

const LIMIT: Duration = Duration::from_millis(200);
const HUNG: Duration = Duration::from_secs(5);

fn fault_of(body: &'static str, listed: usize) -> Result<Option<FunctionFault>, &'static str> {
    let (sent, received) = mpsc::channel();
    thread::spawn(move || {
        let objects = vec![every_field_task(); listed];
        let fault = run_filter_limited(body, &objects, LIMIT)
            .err()
            .map(|failure| failure.fault);
        let _ = sent.send(fault);
    });
    received
        .recv_timeout(HUNG)
        .map_err(|_| "match ran on past the limit")
}

fn stopped(body: &'static str) {
    assert_eq!(
        fault_of(body, 1),
        Ok(Some(FunctionFault::RanOver)),
        "{body}"
    );
}

#[test]
fn a_loop_inside_a_coroutine_is_stopped_at_the_limit() {
    stopped("local co = coroutine.wrap(function() while true do end end)\nco()");
}

#[test]
fn a_pcall_in_a_loop_cannot_swallow_the_limit() {
    stopped("while true do pcall(function() while true do end end) end");
}

#[test]
fn a_pcall_in_a_loop_inside_a_coroutine_cannot_swallow_the_limit() {
    stopped(
        "coroutine.wrap(function()\n\
         while true do pcall(function() while true do end end) end\n\
         end)()",
    );
}

#[test]
fn an_xpcall_in_a_loop_cannot_swallow_the_limit() {
    stopped(
        "while true do\n\
         xpcall(function() while true do end end, function(e) return e end)\n\
         end",
    );
}

#[test]
fn a_coroutine_resumed_in_a_loop_is_stopped_at_the_limit() {
    stopped(
        "local co = coroutine.create(function() while true do end end)\n\
         while true do coroutine.resume(co) end",
    );
}

#[test]
fn a_close_handler_that_loops_is_stopped_at_the_limit() {
    stopped(
        "do\n\
         local guard <close> = setmetatable({}, { __close = function() while true do end end })\n\
         end\n\
         return true",
    );
}

#[test]
fn a_coroutine_kept_in_a_global_and_resumed_by_a_later_match_is_stopped() {
    let body = "if kept == nil then\n\
                kept = coroutine.wrap(function() coroutine.yield() while true do end end)\n\
                kept()\n\
                return true\n\
                end\n\
                kept()";
    assert_eq!(fault_of(body, 2), Ok(Some(FunctionFault::RanOver)));
}
