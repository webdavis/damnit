use dam_application::BackgroundJob;

pub(super) fn plist(job: &BackgroundJob) -> String {
    let arguments: String = std::iter::once(job.dam.clone())
        .chain(job.arguments())
        .map(|word| format!("\n    <string>{}</string>", escaped(&word)))
        .collect();
    let label = escaped(&job.label());
    let search_path = escaped(&job.search_path);
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
