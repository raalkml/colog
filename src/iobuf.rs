extern crate libc;

pub struct IoBuf<const CAPACITY: usize> {
    buffer: [u8; CAPACITY],
    begin: usize,
    end: usize,
}

impl<const CAPACITY: usize> IoBuf<CAPACITY> {
    pub fn new() -> IoBuf<CAPACITY> {
        IoBuf { buffer: [0; CAPACITY], begin: 0, end: 0 }
    }
    pub fn get_buf(&mut self, n: usize) -> &mut [u8] {
        let end;
        if self.end < CAPACITY {
            end = CAPACITY.min(self.end + n);
        } else if self.begin > 0 {
            self.buffer[..].copy_within(self.begin .. self.end, 0);
            self.end -= self.begin;
            self.begin = 0;
            end = CAPACITY.min(self.end + n);
        } else {
            // completely full, return an empty slice
            end = self.end
        }
        &mut self.buffer[self.end .. end]
    }
    pub fn put_buf(&mut self, n: usize) {
        self.end += n;
    }

    pub fn peek(&self) -> &[u8] {
        &self.buffer[self.begin..self.end]
    }
    pub fn peek_line(&self) -> &[u8] {
        for i in self.begin..self.end {
            if self.buffer[i] == 0x0Au8 {
                return &self.buffer[self.begin .. i + 1];
            }
        }
        &self.buffer[self.begin..self.begin]
    }
    pub fn consume(&mut self, n: usize) { self.begin += n; }
}

#[cfg(test)]
mod tests;

