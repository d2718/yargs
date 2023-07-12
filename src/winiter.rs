/**!
Like the iterator in `iter`, but slightly different due to Win64 quirks.

While this still matches using _bytes_, the returned chunks must be
valid UTF-8.
*/
use std::io::{ErrorKind, Read};

// RegexChunker read buffer size is 1 KiB.
const READ_BUFF_SIZE: usize = 1024;

use regex::bytes::Regex;

pub struct RegexChunker<R> {
    fence: Regex,
    read_buff: [u8; READ_BUFF_SIZE],
    search_buff: Vec<u8>,
    source: R,
    has_errored: bool,
    no_match_in_buff: bool,
}

impl<R> RegexChunker<R> {
    pub fn new(source: R, fence: &str) -> Result<Self, String> {
        let fence = Regex::new(fence)
            .map_err(|e| format!("Invalid regex pattern \"{}\": {}", fence, &e))?;

        Ok(Self {
            fence, source,
            read_buff: [0u8; READ_BUFF_SIZE],
            search_buff: Vec::new(),
            has_errored: false,
            no_match_in_buff: true,
        })
    }

    fn scan_search_buffer(&mut self) -> Option<Result<String, String>> {
        let (start, end) = match self.fence.find(&self.search_buff) {
            Some(m) => {
                self.no_match_in_buff = false;
                (m.start(), m.end())
            },
            None => {
                self.no_match_in_buff = true;
                return None;
            }
        };

        let mut new_buff = self.search_buff.split_off(end);
        self.search_buff.resize(start, 0);
        std::mem::swap(&mut new_buff, &mut self.search_buff);

        match String::from_utf8(new_buff) {
            Ok(chunk) => Some(Ok(chunk)),
            Err(e) => {
                self.has_errored = true;
                Some(Err(format!("input not valid UTF-8: {}", &e)))
            }
        }
    }
}

impl<R: Read> Iterator for RegexChunker<R> {
    type Item = Result<String, String>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.has_errored {
            return None;
        }

        loop {
            if self.no_match_in_buff {
                match self.source.read(&mut self.read_buff) {
                    Err(e) => match e.kind() {
                        ErrorKind::WouldBlock | ErrorKind::Interrupted => {
                            std::hint::spin_loop();
                            continue;
                        }
                        _ => {
                            let errmsg = format!("error reading input: {}", &e);
                            self.has_errored = true;
                            return Some(Err(errmsg));
                        },
                    },
                    Ok(0) => {
                        if self.search_buff.is_empty() {
                            return None;
                        } else {
                            let mut new_buff: Vec<u8> = Vec::new();
                            std::mem::swap(&mut self.search_buff, &mut new_buff);
                            match String::from_utf8(new_buff) {
                                Ok(chunk) => return Some(Ok(chunk)),
                                Err(e) => {
                                    self.has_errored = true;
                                    let errmsg = format!(
                                        "input not valid UTF-8: {}", &e
                                    );
                                    return Some(Err(errmsg));
                                },
                            }
                        }
                    },
                    Ok(n) => {
                        self.search_buff.extend_from_slice(&self.read_buff[..n]);
                        match self.scan_search_buffer() {
                            Some(rval) => return Some(rval),
                            None => {
                                std::hint::spin_loop();
                                continue;
                            }
                        }
                    },
                }
            } else {
                match self.scan_search_buffer() {
                    Some(rval) => return Some(rval),
                    None => {
                        std::hint::spin_loop();
                        continue;
                    }
                }
            }
        }
    }
}