use super::fields::select_fields;

pub fn closest_field(typed: &str) -> Option<&'static str> {
    select_fields()
        .iter()
        .map(|f| (distance(typed, f.key), f.key))
        .filter(|(d, key)| *d <= key.len() / 3 + 1)
        .min_by_key(|(d, _)| *d)
        .map(|(_, key)| key)
}

fn distance(a: &str, b: &str) -> usize {
    let b: Vec<char> = b.chars().collect();
    let mut row: Vec<usize> = (0..=b.len()).collect();
    for (i, ca) in a.chars().enumerate() {
        let mut diagonal = row[0];
        row[0] = i + 1;
        for (j, cb) in b.iter().enumerate() {
            let next = (diagonal + usize::from(ca != *cb))
                .min(row[j] + 1)
                .min(row[j + 1] + 1);
            diagonal = row[j + 1];
            row[j + 1] = next;
        }
    }
    row[b.len()]
}
