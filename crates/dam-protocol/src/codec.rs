use std::io::{self, BufRead, Write};

use serde::Serialize;
use serde::de::DeserializeOwned;

/// One JSON document per line, newline terminated.
pub fn write_line<T: Serialize>(out: &mut impl Write, value: &T) -> io::Result<()> {
    serde_json::to_writer(&mut *out, value).map_err(io::Error::other)?;
    out.write_all(b"\n")?;
    out.flush()
}

/// The most bytes one line may hold, its newline included. A helper that
/// sends a stream with no newline in it would otherwise grow the read buffer
/// until the process dies, which the request deadline does not bound.
pub const MAX_LINE: u64 = 16 * 1024 * 1024;

/// The next non-blank line as `T`; `None` at end of input. A line that is
/// not `T`, is not UTF-8, or is longer than `MAX_LINE` is `InvalidData`.
pub fn read_line<T: DeserializeOwned>(input: &mut impl BufRead) -> io::Result<Option<T>> {
    read_line_bounded(input, MAX_LINE)
}

/// `read_line` with the limit named, so the threshold is tested without
/// building a line of the shipped size.
fn read_line_bounded<T: DeserializeOwned>(
    input: &mut impl BufRead,
    max: u64,
) -> io::Result<Option<T>> {
    let mut line = Vec::new();
    loop {
        line.clear();
        // A fresh limit per line: the budget bounds one line, not the stream.
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
mod tests {
    use super::*;
    use crate::Request;
    use std::io::Cursor;

    #[test]
    fn write_then_read_one_line_each() {
        let mut buf = Vec::new();
        write_line(&mut buf, &Request::Capabilities).unwrap();
        write_line(&mut buf, &Request::Pull { since: None }).unwrap();
        assert_eq!(
            buf,
            b"{\"cmd\":\"capabilities\"}\n{\"cmd\":\"pull\",\"since\":null}\n"
        );
        let mut input = Cursor::new(buf);
        assert_eq!(
            read_line::<Request>(&mut input).unwrap(),
            Some(Request::Capabilities)
        );
        assert_eq!(
            read_line::<Request>(&mut input).unwrap(),
            Some(Request::Pull { since: None })
        );
        assert_eq!(read_line::<Request>(&mut input).unwrap(), None);
    }

    #[test]
    fn a_bad_line_is_an_invalid_data_error_not_a_panic() {
        let mut input = Cursor::new(b"not json\n".to_vec());
        let err = read_line::<Request>(&mut input).unwrap_err();
        assert_eq!(err.kind(), std::io::ErrorKind::InvalidData);
    }

    #[test]
    fn a_line_exactly_at_the_limit_including_its_newline_decodes() {
        let doc = b"{\"cmd\":\"capabilities\"}\n";
        let mut input = Cursor::new(doc.to_vec());
        assert_eq!(
            read_line_bounded::<Request>(&mut input, doc.len() as u64).unwrap(),
            Some(Request::Capabilities)
        );
    }

    #[test]
    fn a_line_one_byte_past_the_limit_is_refused_naming_the_limit() {
        let doc = b"{\"cmd\":\"capabilities\"}\n";
        let limit = doc.len() as u64 - 1;
        let mut input = Cursor::new(doc.to_vec());
        let err = read_line_bounded::<Request>(&mut input, limit).unwrap_err();
        assert_eq!(err.kind(), std::io::ErrorKind::InvalidData);
        assert!(err.to_string().contains(&limit.to_string()), "{err}");
    }

    #[test]
    fn a_last_line_with_no_newline_still_decodes_below_the_limit() {
        let doc = b"{\"cmd\":\"capabilities\"}";
        let mut input = Cursor::new(doc.to_vec());
        assert_eq!(
            read_line_bounded::<Request>(&mut input, doc.len() as u64 + 1).unwrap(),
            Some(Request::Capabilities)
        );
    }

    #[test]
    fn each_line_gets_its_own_budget_rather_than_one_shared_across_the_stream() {
        let doc = b"{\"cmd\":\"capabilities\"}\n{\"cmd\":\"capabilities\"}\n";
        let mut input = Cursor::new(doc.to_vec());
        let limit = doc.len() as u64 / 2;
        assert!(
            read_line_bounded::<Request>(&mut input, limit)
                .unwrap()
                .is_some()
        );
        assert!(
            read_line_bounded::<Request>(&mut input, limit)
                .unwrap()
                .is_some()
        );
    }

    #[test]
    fn a_helper_that_never_sends_a_newline_is_refused_instead_of_buffered_without_bound() {
        let mut endless = std::io::BufReader::new(std::io::repeat(b'x'));
        let err = read_line::<Request>(&mut endless).unwrap_err();
        assert_eq!(err.kind(), std::io::ErrorKind::InvalidData);
        assert!(err.to_string().contains(&MAX_LINE.to_string()), "{err}");
    }

    #[test]
    fn blank_lines_are_skipped() {
        let mut input = Cursor::new(b"\n\n{\"cmd\":\"capabilities\"}\n".to_vec());
        assert_eq!(
            read_line::<Request>(&mut input).unwrap(),
            Some(Request::Capabilities)
        );
    }
}
