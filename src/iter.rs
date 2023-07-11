/**!
An iterator over chunks of an input stream delimited by an arbitrary regex.
*/
use std::{
    ffi::OsString,
    io::{ErrorKind, Read},
    os::unix::ffi::OsStringExt,
};

// RegexChunker read buffer size is 1 KiB.
const READ_BUFF_SIZE: usize = 1024;

use regex::bytes::Regex;

/**
The iterator. It wraps a `Read`er and iterates over chunks of its input
delimited by a supplied regex.
*/
pub struct RegexChunker<R> {
    fence: Regex,
    read_buff: [u8; READ_BUFF_SIZE],
    search_buff: Vec<u8>,
    source: R,
    has_errored: bool,
    no_match_in_buff: bool,
}

impl<R> RegexChunker<R> {
    /// Wrap `source`, and chunk its output based on `fence`.
    pub fn new(source: R, fence: &str) -> Result<Self, String> {
        let fence = Regex::new(fence)
            .map_err(|e| format!("invalid regex pattern \"{}\": {}", fence, &e))?;

        Ok(Self {
            fence,
            read_buff: [0u8; READ_BUFF_SIZE],
            search_buff: Vec::new(),
            source,
            has_errored: false,
            no_match_in_buff: true,
        })
    }

    // Attempts to find the first match in the search_buffer, returning the
    // portion before it, and leaving the portion after it in the buffer.
    fn scan_search_buffer(&mut self) -> Option<Vec<u8>> {
        let (start, end) = match self.fence.find(&self.search_buff) {
            Some(m) => {
                self.no_match_in_buff = false;
                (m.start(), m.end())
            }
            None => {
                self.no_match_in_buff = true;
                return None;
            }
        };

        let mut new_buff = self.search_buff.split_off(end);
        self.search_buff.resize(start, 0);
        std::mem::swap(&mut new_buff, &mut self.search_buff);
        self.no_match_in_buff = false;
        return Some(new_buff);
    }
}

impl<R: Read> Iterator for RegexChunker<R> {
    type Item = Result<OsString, String>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.has_errored {
            return None;
        }

        'reader: loop {
            if self.no_match_in_buff {
                match self.source.read(&mut self.read_buff) {
                    Err(e) => match e.kind() {
                        ErrorKind::WouldBlock | ErrorKind::Interrupted => {
                            std::hint::spin_loop();
                            continue 'reader;
                        }
                        _ => {
                            let errmsg = format!("error reading input: {}", &e);
                            self.has_errored = true;
                            return Some(Err(errmsg));
                        }
                    },
                    Ok(0) => {
                        if self.search_buff.is_empty() {
                            return None;
                        } else {
                            let mut new_buff: Vec<u8> = Vec::new();
                            std::mem::swap(&mut self.search_buff, &mut new_buff);
                            return Some(Ok(OsString::from_vec(new_buff)));
                        }
                    }
                    Ok(n) => {
                        self.search_buff.extend_from_slice(&self.read_buff[..n]);
                        match self.scan_search_buffer() {
                            Some(v) => return Some(Ok(OsString::from_vec(v))),
                            None => {
                                std::hint::spin_loop();
                                continue 'reader;
                            }
                        }
                    }
                }
            } else {
                match self.scan_search_buffer() {
                    Some(v) => return Some(Ok(OsString::from_vec(v))),
                    None => {
                        std::hint::spin_loop();
                        continue 'reader;
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn split_on_caps() {
        let f = std::fs::File::open("test/cheese.txt").unwrap();
        let chunker = RegexChunker::new(f, r"[A-Z]").unwrap();

        let mut buff = String::new();
        for chunk in chunker {
            let chstr = chunk.unwrap();
            let ch = chstr.as_os_str().to_str().unwrap();
            buff.push_str(ch);
            buff.push('\n');
        }

        let chunked_text = std::fs::read_to_string("test/cheese_chunked.txt").unwrap();
        assert_eq!(buff.as_str(), chunked_text.as_str());
    }
}
