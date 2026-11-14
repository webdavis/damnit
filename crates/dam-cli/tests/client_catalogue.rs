mod support;

use support::sandbox::Sandbox;

#[test]
fn category_list_and_filter_list_print_what_config_declares() {
    let _guard = support::guard("category_list_and_filter_list_print_what_config_declares");
    let sb = Sandbox::new();
    sb.add_category("effort", &["light", "admin", "deep"], true);
    sb.add_category("context", &["home", "errand"], false);
    sb.add_filter("today", "due:today | overdue");

    let (ok, out, err) = sb.dam(&["category", "list", "--json"]);
    assert!(ok, "{err}");
    let answer: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(
        answer,
        serde_json::json!({"categories": [
            {"name": "context", "values": ["home", "errand"], "exclusive": false},
            {"name": "effort", "values": ["light", "admin", "deep"], "exclusive": true},
        ]}),
        "{answer}"
    );

    let (ok, out, err) = sb.dam(&["filter", "list", "--json"]);
    assert!(ok, "{err}");
    let answer: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(
        answer,
        serde_json::json!({"filters": [{"name": "today", "query": "due:today | overdue"}]}),
        "{answer}"
    );

    let (ok, out, err) = sb.dam(&["category", "list"]);
    assert!(ok, "{err}");
    assert!(out.contains("effort"), "{out}");
    assert!(out.contains("light, admin, deep"), "{out}");
    let (ok, out, err) = sb.dam(&["filter", "list"]);
    assert!(ok, "{err}");
    assert!(out.contains("due:today | overdue"), "{out}");
}

#[test]
fn an_empty_catalogue_is_an_empty_array() {
    let _guard = support::guard("an_empty_catalogue_is_an_empty_array");
    let sb = Sandbox::new();

    for (args, key) in [
        (["category", "list", "--json"], "categories"),
        (["filter", "list", "--json"], "filters"),
    ] {
        let (ok, out, err) = sb.dam(&args);
        assert!(ok, "{err}");
        let answer: serde_json::Value = serde_json::from_str(&out).unwrap();
        assert_eq!(answer[key], serde_json::json!([]), "{answer}");
    }
}

#[test]
fn the_catalogue_renders_as_a_toon_table_per_list() {
    let _guard = support::guard("the_catalogue_renders_as_a_toon_table_per_list");
    let sb = Sandbox::new();
    sb.add_filter("today", "due:today");

    let (ok, out, err) = sb.dam(&["filter", "list", "--toon"]);
    assert!(ok, "{err}");
    assert!(out.starts_with("filters[1]{name,query}:"), "{out}");
}

#[test]
fn the_catalogue_answers_although_the_store_will_not_open() {
    let _guard = support::guard("the_catalogue_answers_although_the_store_will_not_open");
    let sb = Sandbox::new();
    sb.add_category("effort", &["deep"], true);
    sb.add_filter("today", "due:today");

    let a_directory_where_the_store_file_goes = sb.dir.path().join("dam.db");
    std::fs::create_dir(&a_directory_where_the_store_file_goes).unwrap();

    let broken = sb.output(&["ls", "--json"]);
    assert_eq!(
        broken.status.code(),
        Some(1),
        "the store opened after all, so this proves nothing: {}",
        String::from_utf8_lossy(&broken.stderr)
    );

    for (args, key, name) in [
        (["category", "list", "--json"], "categories", "effort"),
        (["filter", "list", "--json"], "filters", "today"),
    ] {
        let out = sb.output(&args);
        assert_eq!(
            out.status.code(),
            Some(0),
            "{args:?} reached the store: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        let answer: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
        assert_eq!(answer[key][0]["name"], name, "{answer}");
    }
}
