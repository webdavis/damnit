use super::*;

fn words(line: &str) -> Vec<String> {
    line.split_whitespace().map(str::to_string).collect()
}

#[test]
fn the_client_id_is_the_one_flag() {
    assert_eq!(
        client_id(&words("--client-id 123.apps")),
        Ok("123.apps".to_string())
    );
    assert!(client_id(&words("")).unwrap_err().contains("--client-id"));
    assert!(
        client_id(&words("--client-id"))
            .unwrap_err()
            .contains("--client-id")
    );
    assert!(
        client_id(&words("--client-id a --verbose"))
            .unwrap_err()
            .contains("--verbose")
    );
}

#[test]
fn a_stray_argument_that_may_be_a_pasted_secret_is_refused_without_being_quoted() {
    let said = client_id(&words("--client-id a GOCSPX-stray")).unwrap_err();
    assert!(!said.contains("GOCSPX-stray"), "{said}");
    assert!(
        said.contains("an argument this walk does not take"),
        "{said}"
    );
}

#[test]
fn a_client_secret_on_the_command_line_is_refused_by_name() {
    for line in [
        "--client-id a --client-secret s",
        "--client-id a --client-secret=s",
    ] {
        let said = client_id(&words(line)).unwrap_err();
        assert!(said.contains("standard input"), "{said}");
        assert!(!said.contains("=s"), "{said}");
    }
}

#[test]
fn the_secret_is_read_from_a_pipe_and_refused_from_a_terminal() {
    assert_eq!(
        client_secret("GOCSPX-abc\n".as_bytes(), false)
            .unwrap()
            .expose(),
        "GOCSPX-abc"
    );
    assert!(
        client_secret("GOCSPX-abc\n".as_bytes(), true)
            .unwrap_err()
            .contains("pipe")
    );
    assert!(
        client_secret(" \n".as_bytes(), false)
            .unwrap_err()
            .contains("no client secret")
    );
}
