use super::*;

fn choose_with_answers(answers: &[&str], options: &[&str]) -> (Result<usize, CliError>, String) {
    let mut err = Vec::new();
    let mut queue: Vec<String> = answers.iter().rev().map(|a| format!("{a}\n")).collect();
    let mut answer = || queue.pop().ok_or(CliError::Cancelled);
    let chosen = choose_asking_once(&mut err, &mut answer, "Which one?", options);
    (chosen, String::from_utf8(err).unwrap())
}

#[test]
fn a_bad_answer_asks_for_a_number_and_never_repeats_the_question() {
    let (chosen, said) = choose_with_answers(&["7", "x", "1"], &["up", "into", "keep"]);
    assert_eq!(chosen.unwrap(), 0);
    assert_eq!(
        said,
        "Which one?\n  1) up\n  2) into\n  3) keep\n> answer 1, 2 or 3\n> answer 1, 2 or 3\n> "
    );
}

#[test]
fn the_hint_reads_naturally_for_one_two_and_many_options() {
    assert_eq!(one_of_the_numbers(1), "1");
    assert_eq!(one_of_the_numbers(2), "1 or 2");
    assert_eq!(one_of_the_numbers(4), "1, 2, 3 or 4");
}

#[test]
fn an_answer_that_never_comes_cancels() {
    let (chosen, said) = choose_with_answers(&[], &["up", "keep"]);
    assert!(matches!(chosen, Err(CliError::Cancelled)));
    assert_eq!(said, "Which one?\n  1) up\n  2) keep\n> ");
}
