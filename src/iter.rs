/**!
The `DelimIterator` type returns chunks delimited by an arbitrary zigamorph.
*/
use std::{
    ffi::OsString,
    io::{BufRead, ErrorKind},
    os::unix::ffi::OsStringExt,
};

pub struct DelimIterator<B> {
    fence: Vec<u8>,
    buff: Vec<u8>,
    source: B,
    total: usize,
    n_reads: usize,
    mean: f64,
    pseudoerr: f64,
    has_errored: bool,
}

impl<B> DelimIterator<B> {
    /// Returns a new DelimIterator, wrapping the source, with the
    /// supplied fence.
    pub fn new(source: B, fence: Vec<u8>) -> Self {
        Self {
            fence: fence.into_iter().rev().collect(),
            buff: Vec::new(),
            source,
            total: 0,
            n_reads: 0,
            mean: 0.0,
            pseudoerr: 0.0,
            has_errored: false,
        }
    }
    
    fn new_buffer(&self) -> Vec<u8> {
        if self.n_reads == 0 {
            return Vec::new();
        }
        let stdev = self.pseudoerr.sqrt() / (self.n_reads as f64);
        let buff_size = (self.mean + (1.64 * stdev)) as usize;
        return Vec::with_capacity(buff_size);
    }
}

impl<B: BufRead> Iterator for DelimIterator<B> {
    type Item = Result<OsString, String>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.has_errored {
            return None;
        }
        
        'reader: loop {
            match self.source.read_until(self.fence[0], &mut self.buff) {
                Err(e) => {
                    match e.kind() {
                        ErrorKind::WouldBlock | ErrorKind::Interrupted => {
                            std::hint::spin_loop();
                            continue;
                        },
                        _ => {
                            let errmsg = format!("error reading input: {}", &e);
                            self.has_errored = true;
                            return Some(Err(errmsg));
                        },
                    }
                },
                Ok(0) => {
                    if self.buff.is_empty() {
                        return None;
                    } else {
                        let mut new_buff = self.new_buffer();
                        std::mem::swap(&mut self.buff, &mut new_buff);
                        return Some(Ok(OsString::from_vec(new_buff)));
                    }
                },
                Ok(n) => {
                    self.total += n;
                    self.n_reads += 1;
                    self.mean = (self.total as f64) / (self.n_reads as f64);
                    self.pseudoerr += (n as f64 - self.mean).powf(2.0);

                    if n < self.fence.len() {
                        continue;
                    }

                    for (n, &b) in self.fence.iter().enumerate() {
                        if b != self.buff[self.buff.len()-n-1] {
                            continue 'reader;
                        }
                    }
                    
                    // Removing the zigamorph.
                    for _ in (0..self.fence.len()).into_iter() {
                        _ = self.buff.pop();
                    }

                    let mut new_buff = self.new_buffer();
                    std::mem::swap(&mut self.buff, &mut new_buff);
                    return Some(Ok(OsString::from_vec(new_buff)));
                },
            }
        }       
    }                
}