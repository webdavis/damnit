pub fn error_document(out: &std::process::Output) -> serde_json::Value {
    let err = String::from_utf8_lossy(&out.stderr);
    serde_json::from_str(&err)
        .unwrap_or_else(|e| panic!("stderr is not one JSON document: {e}\n{err}"))
}

pub fn kind_of(out: &std::process::Output) -> String {
    error_document(out)["error"]["kind"]
        .as_str()
        .unwrap_or_default()
        .to_string()
}
