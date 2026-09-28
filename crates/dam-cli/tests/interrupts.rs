mod support;

use std::process::Command;

use support::sandbox::Sandbox;

const A_LOADED_MACHINES_MARGIN: std::time::Duration = std::time::Duration::from_secs(10);

#[test]
fn an_interrupt_during_a_helper_exchange_exits_three_and_kills_the_helper() {
    let _guard =
        support::guard("an_interrupt_during_a_helper_exchange_exits_three_and_kills_the_helper");
    let sb = Sandbox::new();
    sb.install_silent_helper();
    let mut dam = sb.spawn(&["push"]);
    let waited = std::time::Instant::now();
    while !sb.helper_running() {
        assert!(
            waited.elapsed() < A_LOADED_MACHINES_MARGIN,
            "the helper never started"
        );
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    assert!(
        Command::new("kill")
            .args(["-INT", &dam.id().to_string()])
            .status()
            .unwrap()
            .success()
    );
    let status = dam.wait().unwrap();
    assert_eq!(status.code(), Some(3), "{status:?}");
    assert!(
        sb.no_helper_left_within(A_LOADED_MACHINES_MARGIN),
        "the helper outlived dam"
    );
    let err = stderr_read_once_the_helper_inheriting_it_is_gone(&mut dam);
    assert!(err.trim().ends_with("cancelled"), "{err}");
}

fn stderr_read_once_the_helper_inheriting_it_is_gone(dam: &mut std::process::Child) -> String {
    let mut err = String::new();
    std::io::Read::read_to_string(&mut dam.stderr.take().unwrap(), &mut err).unwrap();
    err
}

fn exit_within_a_second_or_fail_rather_than_hang(
    child: &mut std::process::Child,
) -> std::process::ExitStatus {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(1);
    while std::time::Instant::now() < deadline {
        if let Some(status) = child.try_wait().unwrap() {
            return status;
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    let _ = child.kill();
    let _ = child.wait();
    panic!("dam was still running a second after the interrupt");
}

struct Drained {
    text: std::sync::Arc<std::sync::Mutex<String>>,
    reader: std::thread::JoinHandle<()>,
}

impl Drained {
    fn so_far(&self) -> String {
        self.text.lock().unwrap().clone()
    }

    fn through_end_of_input(self) -> String {
        self.reader.join().unwrap();
        self.text.lock().unwrap().clone()
    }
}

fn drained_on_a_thread_so_a_full_pipe_cannot_deadlock(
    mut pipe: impl std::io::Read + Send + 'static,
) -> Drained {
    let text = std::sync::Arc::new(std::sync::Mutex::new(String::new()));
    let sink = std::sync::Arc::clone(&text);
    let reader = std::thread::spawn(move || {
        let mut buf = [0u8; 256];
        while let Ok(n) = pipe.read(&mut buf) {
            if n == 0 {
                return;
            }
            sink.lock()
                .unwrap()
                .push_str(&String::from_utf8_lossy(&buf[..n]));
        }
    });
    Drained { text, reader }
}

#[test]
fn an_interrupt_at_a_prompt_exits_three_instead_of_waiting_for_an_answer() {
    let _guard =
        support::guard("an_interrupt_at_a_prompt_exits_three_instead_of_waiting_for_an_answer");
    let sb = Sandbox::new();
    let parent = sb.new_object(&["parent"]);
    assert!(sb.dam(&["new", "child", "--path", "parent/"]).0);
    let mut dam = sb.spawn_with_stdin(
        &["done", &parent[..7], "--force", "--interactive"],
        std::process::Stdio::piped(),
    );
    let _stdin_held_open_so_the_prompt_never_sees_end_of_input = dam.stdin.take().unwrap();
    let asked = drained_on_a_thread_so_a_full_pipe_cannot_deadlock(dam.stderr.take().unwrap());
    let waited = std::time::Instant::now();
    while !asked.so_far().contains("open child task(s)") {
        assert!(
            waited.elapsed() < A_LOADED_MACHINES_MARGIN,
            "the prompt never appeared: {}",
            asked.so_far()
        );
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    assert!(
        Command::new("kill")
            .args(["-INT", &dam.id().to_string()])
            .status()
            .unwrap()
            .success()
    );
    let status = exit_within_a_second_or_fail_rather_than_hang(&mut dam);
    assert_eq!(status.code(), Some(3), "{status:?}");
    let said = asked.through_end_of_input();
    assert!(said.trim().ends_with("cancelled"), "{said}");
}
