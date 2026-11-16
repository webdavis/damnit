use super::*;
use crate::piped::with_piped_ids;

#[test]
fn ids_piped_in_inside_a_project_are_acted_on_as_given() {
    for verb in VERBS {
        let ctx = holding_a_project_and_more();
        let rows = format!("{}\n{}\n", oid_of(&ctx, "readme"), oid_of(&ctx, "laundry"));
        let words: Vec<&str> = [verb, &["-", "--preview"]].concat();
        let typed: Vec<String> = words.iter().map(|w| w.to_string()).collect();
        let cli = Cli::try_parse_from(std::iter::once("dam").chain(words)).unwrap();
        let command = with_piped_ids(cli, &mut rows.as_bytes(), false, &typed)
            .unwrap()
            .unwrap()
            .command;
        let ctx = following(ctx, &command, &inside());
        let report = dispatch_in(&ctx, command, inside()).unwrap();
        assert_eq!(report.data["count"], 2, "dam {verb:?}");
    }
}
