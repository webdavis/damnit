use super::answers::json_of;
use super::fixture;
use super::sandbox::Sandbox;

pub fn answer(command: &str) -> Option<Vec<serde_json::Value>> {
    match command {
        "category create" => Some(vec![json_of(
            &Sandbox::new(),
            &[
                "category",
                "create",
                "place",
                "--values",
                "home,work",
                "--exclusive",
            ],
        )]),
        "category remove" => Some(vec![json_of(
            &fixture::seeded(),
            &["category", "remove", "effort"],
        )]),
        "category migrate" => Some(vec![json_of(
            &fixture::seeded(),
            &["category", "migrate", "--map", "deep=focus", "--preview"],
        )]),
        "filter create" => Some(vec![json_of(
            &Sandbox::new(),
            &["filter", "create", "soon", "--query", "due:today"],
        )]),
        "filter remove" => Some(vec![json_of(
            &fixture::seeded(),
            &["filter", "remove", "today"],
        )]),
        _ => None,
    }
}
