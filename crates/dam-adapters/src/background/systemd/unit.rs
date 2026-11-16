use dam_application::BackgroundJob;

pub(super) fn service(job: &BackgroundJob) -> String {
    let command: Vec<String> = std::iter::once(job.dam.clone())
        .chain(job.arguments())
        .map(|word| quoted(&word))
        .collect();
    let remote = plain(&job.remote);
    let search_path = quoted(&format!("PATH={}", job.search_path));
    format!(
        "[Unit]\nDescription=dam sync run {remote}\n\n[Service]\nType=oneshot\nEnvironment={search_path}\nExecStart={}\n",
        command.join(" ")
    )
}

pub(super) fn timer(remote: &str) -> String {
    let remote = plain(remote);
    format!(
        "[Unit]\nDescription=dam sync run {remote} every minute\n\n[Timer]\nOnCalendar=minutely\n\n[Install]\nWantedBy=timers.target\n"
    )
}

fn plain(text: &str) -> String {
    text.replace('%', "%%").replace(['\n', '\r'], " ")
}

fn quoted(word: &str) -> String {
    let escaped = plain(word)
        .replace('\\', r"\\")
        .replace('"', "\\\"")
        .replace('$', "$$");
    format!("\"{escaped}\"")
}
