use std::io::{self, BufRead, Write};

use serde::Serialize;
use serde::de::DeserializeOwned;

/// One JSON document per line, newline terminated.
pub fn write_line<T: Serialize>(out: &mut impl Write, value: &T) -> io::Result<()> {
    serde_json::to_writer(&mut *out, value).map_err(io::Error::other)?;
    out.write_all(b"\n")?;
    out.flush()
}

/// The next non-blank line as `T`; `None` at end of input. A line that is
/// not `T` is `InvalidData` with the parser's message.
pub fn read_line<T: DeserializeOwned>(input: &mut impl BufRead) -> io::Result<Option<T>> {
    let mut line = String::new();
    loop {
        line.clear();
        if input.read_line(&mut line)? == 0 {
            return Ok(None);
        }
        if line.trim().is_empty() {
            continue;
        }
        return serde_json::from_str(line.trim_end())
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
    fn blank_lines_are_skipped() {
        let mut input = Cursor::new(b"\n\n{\"cmd\":\"capabilities\"}\n".to_vec());
        assert_eq!(
            read_line::<Request>(&mut input).unwrap(),
            Some(Request::Capabilities)
        );
    }
}
