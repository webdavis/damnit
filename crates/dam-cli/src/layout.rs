const GAP: &str = "  ";
const EMPTY_CELL: &str = "-";
const DEFAULT_WIDTH: usize = 80;

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

pub(crate) fn fit(text: &str, width: usize) -> String {
    text.lines()
        .map(|line| {
            if line.chars().count() <= width {
                line.to_string()
            } else {
                let kept: String = line.chars().take(width.saturating_sub(1)).collect();
                format!("{kept}…")
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

pub(crate) fn terminal_width(columns: Option<&str>) -> usize {
    columns
        .and_then(|c| c.trim().parse().ok())
        .filter(|&w: &usize| w > 0)
        .unwrap_or(DEFAULT_WIDTH)
}

pub(crate) fn counted(n: usize, one: &str, many: &str) -> String {
    format!("{n} {}", if n == 1 { one } else { many })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cells(row: &[&str]) -> Vec<String> {
        row.iter().map(|c| c.to_string()).collect()
    }

    #[test]
    fn every_column_but_the_last_pads_to_its_widest_cell() {
        let rows = vec![
            cells(&["37cfce3", "event", "2026-10-13", "", "offsite"]),
            cells(&["48cc07f", "p2", "", "admin/", "file taxes  [admin, money]"]),
        ];
        assert_eq!(
            table(&rows),
            vec![
                "37cfce3  event  2026-10-13  -       offsite",
                "48cc07f  p2     -           admin/  file taxes  [admin, money]",
            ]
        );
    }

    #[test]
    fn a_line_past_the_width_is_cut_with_an_ellipsis_and_a_short_one_is_kept() {
        assert_eq!(fit("abcdef\nabc", 4), "abc…\nabc");
        assert_eq!(fit("abcd", 4), "abcd");
    }

    #[test]
    fn the_width_is_columns_or_eighty() {
        assert_eq!(terminal_width(Some("40")), 40);
        assert_eq!(terminal_width(None), 80);
        assert_eq!(terminal_width(Some("0")), 80);
        assert_eq!(terminal_width(Some("wide")), 80);
    }

    #[test]
    fn a_count_names_one_thing_or_many() {
        assert_eq!(counted(1, "commit", "commits"), "1 commit");
        assert_eq!(counted(0, "commit", "commits"), "0 commits");
        assert_eq!(
            counted(2, "open dependency", "open dependencies"),
            "2 open dependencies"
        );
    }
}
