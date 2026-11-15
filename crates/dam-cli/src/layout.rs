use dam_domain::Timestamp;

const GAP: &str = "  ";
const EMPTY_CELL: &str = "-";

pub(crate) fn table(rows: &[Vec<String>]) -> Vec<String> {
    let cell = |c: &String| -> String {
        if c.is_empty() {
            EMPTY_CELL.to_string()
        } else {
            c.clone()
        }
    };
    let rows: Vec<Vec<String>> = rows.iter().map(|r| r.iter().map(cell).collect()).collect();
    let mut widths: Vec<usize> = Vec::new();
    for row in &rows {
        for (i, c) in row.iter().enumerate() {
            let w = c.chars().count();
            match widths.get_mut(i) {
                Some(widest) => *widest = (*widest).max(w),
                None => widths.push(w),
            }
        }
    }
    rows.iter()
        .map(|row| {
            let last = row.len().saturating_sub(1);
            row.iter()
                .enumerate()
                .map(|(i, c)| {
                    if i == last {
                        c.clone()
                    } else {
                        format!("{c:<width$}", width = widths[i])
                    }
                })
                .collect::<Vec<_>>()
                .join(GAP)
        })
        .collect()
}

pub(crate) fn fit(text: &str, width: usize, cut: &[bool]) -> String {
    text.lines()
        .zip(cut.iter().copied().chain(std::iter::repeat(false)))
        .map(|(line, cut)| {
            if !cut || line.chars().count() <= width {
                line.to_string()
            } else {
                let kept: String = line.chars().take(width.saturating_sub(1)).collect();
                format!("{kept}…")
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

pub(crate) fn terminal_width(columns: Option<&str>, terminal: Option<usize>) -> Option<usize> {
    columns
        .and_then(|c| c.trim().parse().ok())
        .filter(|&w: &usize| w > 0)
        .or(terminal)
}

pub(crate) fn counted(n: usize, one: &str, many: &str) -> String {
    format!("{n} {}", if n == 1 { one } else { many })
}

pub(crate) fn ago_to_one_unit_and_never_in_the_future(now: Timestamp, then: Timestamp) -> String {
    const MINUTE: i64 = 60;
    const HOUR: i64 = 60 * MINUTE;
    const DAY: i64 = 24 * HOUR;
    let seconds = now.duration_since(then).as_secs().max(0);
    if seconds < MINUTE {
        "just now".to_string()
    } else if seconds < HOUR {
        format!("{}m ago", seconds / MINUTE)
    } else if seconds < DAY {
        format!("{}h ago", seconds / HOUR)
    } else {
        format!("{}d ago", seconds / DAY)
    }
}

#[cfg(test)]
mod tests;
