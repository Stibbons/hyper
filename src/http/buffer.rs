use std::cmp;
use std::io::{self, Read};
use std::ptr;


const INIT_BUFFER_SIZE: usize = 4096;
const MAX_BUFFER_SIZE: usize = 8192 + 4096 * 100;

#[derive(Debug, Default)]
pub struct Buffer {
    vec: Vec<u8>,
    read_pos: usize,
    write_pos: usize,
}

impl Buffer {
    pub fn new() -> Buffer {
        Buffer::default()
    }

    pub fn reset(&mut self) {
        *self = Buffer::new()
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.read_pos - self.write_pos
    }

    #[inline]
    pub fn is_max_size(&self) -> bool {
        self.len() >= MAX_BUFFER_SIZE
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    pub fn bytes(&self) -> &[u8] {
        &self.vec[self.write_pos..self.read_pos]
    }

    #[inline]
    pub fn consume(&mut self, pos: usize) {
        debug_assert!(self.read_pos >= self.write_pos + pos);
        self.write_pos += pos;
        if self.write_pos == self.read_pos {
            self.write_pos = 0;
            self.read_pos = 0;
        }
    }

    pub fn read_from<R: Read>(&mut self, r: &mut R) -> io::Result<usize> {
        self.maybe_reserve();
        let n = try!(r.read(&mut self.vec[self.read_pos..]));
        self.read_pos += n;
        Ok(n)
    }

    #[inline]
    fn maybe_reserve(&mut self) {
        let cap = self.vec.len();
        if cap == 0 {
            trace!("reserving initial {}", INIT_BUFFER_SIZE);
            self.vec = vec![0; INIT_BUFFER_SIZE];
        } else if self.write_pos > 0  && self.read_pos == cap {
            let count = self.read_pos - self.write_pos;
            trace!("moving buffer bytes over by {}", count);
            unsafe {
                ptr::copy(
                    self.vec.as_ptr().offset(self.write_pos as isize),
                    self.vec.as_mut_ptr(),
                    count
                );
            }
            self.read_pos -= count;
            self.write_pos = 0;
        } else if self.read_pos == cap && cap < MAX_BUFFER_SIZE {
            self.vec.reserve(cmp::min(cap * 4, MAX_BUFFER_SIZE) - cap);
            let new = self.vec.capacity() - cap;
            trace!("reserved {}", new);
            unsafe { grow_zerofill(&mut self.vec, new) }
        }
    }
}

#[inline]
unsafe fn grow_zerofill(buf: &mut Vec<u8>, additional: usize) {
    let len = buf.len();
    buf.set_len(len + additional);
    ptr::write_bytes(buf.as_mut_ptr(), 0, buf.len());
}
