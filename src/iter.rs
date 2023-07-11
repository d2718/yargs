/**!
The `DelimIterator` type returns chunks delimited by an arbitrary zigamorph.
*/
use std::io::{BufRead, ErrorKind};

struct DelimIterator<B> {
    fence: Vec<u8>,
    buff: Vec<u8>,
    source: B,
    mean: f64,
    pseudodev: f64,
}

impl<B: BufRead> Iterator for DelimIterator<B> {
    type Item = Result<String, String>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.source.read_until(self.fence[0], &mut self.buff) {
                Err(e) => {
                    match e.kind() {
                        ErrorKind::WouldBlock | ErrorKind::Interrupted => {
                            std::hint::spin_loop();
                            continue;
                        },
                        ErrorKind::UnexpectedEof => {
                            
                        }
                    }
                }
        }
        
        None
    }
}
