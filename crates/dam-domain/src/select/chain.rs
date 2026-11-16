pub fn chain_to_itself<F>(start: &str, reaches: F) -> Option<Vec<String>>
where
    F: Fn(&str) -> Vec<String>,
{
    let mut path = vec![start.to_string()];
    walk(&reaches, &mut path)
}

fn walk(reaches: &dyn Fn(&str) -> Vec<String>, path: &mut Vec<String>) -> Option<Vec<String>> {
    let at = path.last().cloned().unwrap_or_default();
    for next in reaches(&at) {
        if next == path[0] {
            let mut chain = path.clone();
            chain.push(next);
            return Some(chain);
        }
        if path.contains(&next) {
            continue;
        }
        path.push(next);
        if let Some(chain) = walk(reaches, path) {
            return Some(chain);
        }
        path.pop();
    }
    None
}
