use dam_application::ScheduledRun;

pub(super) fn service(run: &ScheduledRun) -> String {
    let command: Vec<String> = std::iter::once(run.dam.as_str())
        .chain(run.arguments())
        .map(quoted)
        .collect();
    let search_path = quoted(&format!("PATH={}", run.search_path));
    let state_home = quoted(&format!("XDG_STATE_HOME={}", run.state_home));
    format!(
        "[Unit]\nDescription=dam sync run\n\n[Service]\nType=oneshot\nEnvironment={search_path} {state_home}\n\
         ExecStart={}\nStandardOutput=null\nStandardError=null\n",
        command.join(" ")
    )
}

pub(super) const TIMER: &str = "[Unit]\nDescription=dam sync run every minute\n\n[Timer]\nOnActiveSec=0\n\
    OnCalendar=minutely\n\n[Install]\nWantedBy=timers.target\n";

fn quoted(word: &str) -> String {
    let escaped = word
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('%', "%%")
        .replace('$', "$$");
    format!("\"{escaped}\"")
}
