use std::io::{self, BufRead, Write};

use serde::Serialize;
use serde::de::DeserializeOwned;

pub fn write_line<T: Serialize>(out: &mut impl Write, value: &T) -> io::Result<()> {
    serde_json::to_writer(&mut *out, value).map_err(io::Error::other)?;
    out.write_all(b"\n")?;
    out.flush()
}

pub const MAX_LINE: u64 = 16 * 1024 * 1024;

pub fn read_line<T: DeserializeOwned>(input: &mut impl BufRead) -> io::Result<Option<T>> {
    read_line_bounded(input, MAX_LINE)
}

fn read_line_bounded<T: DeserializeOwned>(
    input: &mut impl BufRead,
    max: u64,
) -> io::Result<Option<T>> {
    let mut line = Vec::new();
    loop {
        line.clear();
        let mut one_line = io::Read::take(&mut *input, max);
        let read = one_line.read_until(b'\n', &mut line)?;
        if read == 0 {
            return Ok(None);
        }
        if read as u64 == max && !line.ends_with(b"\n") {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("a line longer than the {max} byte limit"),
            ));
        }
        let text = std::str::from_utf8(&line)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;
        if text.trim().is_empty() {
            continue;
        }
        return serde_json::from_str(text.trim_end())
            .map(Some)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()));
    }
}

#[cfg(test)]
mod tests;
