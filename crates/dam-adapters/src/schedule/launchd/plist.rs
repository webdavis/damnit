use dam_application::ScheduledRun;

pub(super) fn plist(label: &str, run: &ScheduledRun) -> String {
    let arguments: String = std::iter::once(run.dam.as_str())
        .chain(run.arguments())
        .map(|word| format!("\n    <string>{}</string>", escaped(word)))
        .collect();
    let search_path = escaped(&run.search_path);
    let state_home = escaped(&run.state_home);
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>Label</key>
  <string>{label}</string>
  <key>ProgramArguments</key>
  <array>{arguments}
  </array>
  <key>EnvironmentVariables</key>
  <dict>
    <key>PATH</key>
    <string>{search_path}</string>
    <key>XDG_STATE_HOME</key>
    <string>{state_home}</string>
  </dict>
  <key>RunAtLoad</key>
  <true/>
  <key>StartInterval</key>
  <integer>60</integer>
</dict>
</plist>
"#
    )
}

fn escaped(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}
