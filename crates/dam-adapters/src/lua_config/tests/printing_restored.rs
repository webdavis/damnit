use dam_application::{CredentialSpec, RemoteName};

use super::support::secrets;

const WHAT_A_FUNCTION_SEES: &str = "return {\n  remote = { todoist = { api_token = function()\n    return io.type(io.stdout) .. io.type(io.stderr) .. io.type(io.write('')) .. tostring(pcall(string.dump, print))\n  end } },\n}\n";

#[test]
fn a_credential_function_prints_through_the_real_functions() {
    let loaded = secrets(WHAT_A_FUNCTION_SEES).loaded();
    let spec = CredentialSpec::Computed {
        remote: RemoteName("todoist".into()),
        name: "api_token".into(),
    };
    assert_eq!(
        loaded.credentials.resolve(&spec).unwrap().expose(),
        "filefilefilefalse"
    );
}
