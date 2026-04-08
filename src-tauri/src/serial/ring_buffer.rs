/// A fixed-size ring buffer for high-speed serial data accumulation.
pub struct RingBuffer {
    buffer: Vec<u8>,
    capacity: usize,
    write_pos: usize,
    read_pos: usize,
    len: usize,
}

impl RingBuffer {
    /// Create a new ring buffer with the given capacity (default 64 KB).
    pub fn new(capacity: usize) -> Self {
        Self {
            buffer: vec![0u8; capacity],
            capacity,
            write_pos: 0,
            read_pos: 0,
            len: 0,
        }
    }

    /// Write data into the buffer. Returns the number of bytes actually written
    /// (may be less than `data.len()` if the buffer is nearly full).
    pub fn write(&mut self, data: &[u8]) -> usize {
        let space = self.available_space();
        let to_write = data.len().min(space);

        for &byte in &data[..to_write] {
            self.buffer[self.write_pos] = byte;
            self.write_pos = (self.write_pos + 1) % self.capacity;
        }
        self.len += to_write;
        to_write
    }

    /// Drain all available data from the buffer and return it.
    pub fn read_all(&mut self) -> Vec<u8> {
        if self.len == 0 {
            return Vec::new();
        }

        let mut result = Vec::with_capacity(self.len);
        for _ in 0..self.len {
            result.push(self.buffer[self.read_pos]);
            self.read_pos = (self.read_pos + 1) % self.capacity;
        }
        self.len = 0;
        result
    }

    /// Number of bytes currently stored.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns `true` if the buffer contains no data.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Number of bytes that can still be written before the buffer is full.
    pub fn available_space(&self) -> usize {
        self.capacity - self.len
    }

    /// Returns `true` if the buffer is completely full.
    pub fn is_full(&self) -> bool {
        self.len == self.capacity
    }
}

impl Default for RingBuffer {
    fn default() -> Self {
        Self::new(64 * 1024) // 64 KB
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_buffer_is_empty() {
        let buf = RingBuffer::new(16);
        assert!(buf.is_empty());
        assert_eq!(buf.len(), 0);
        assert_eq!(buf.available_space(), 16);
        assert!(!buf.is_full());
    }

    #[test]
    fn test_write_and_read_all() {
        let mut buf = RingBuffer::new(16);
        let written = buf.write(&[1, 2, 3, 4, 5]);
        assert_eq!(written, 5);
        assert_eq!(buf.len(), 5);

        let data = buf.read_all();
        assert_eq!(data, vec![1, 2, 3, 4, 5]);
        assert!(buf.is_empty());
    }

    #[test]
    fn test_write_wraps_around() {
        let mut buf = RingBuffer::new(8);
        // Fill 6 bytes
        buf.write(&[1, 2, 3, 4, 5, 6]);
        // Read them out (advances read_pos)
        let _ = buf.read_all();
        // Now write 5 more bytes — this wraps around
        let written = buf.write(&[7, 8, 9, 10, 11]);
        assert_eq!(written, 5);
        let data = buf.read_all();
        assert_eq!(data, vec![7, 8, 9, 10, 11]);
    }

    #[test]
    fn test_write_partial_when_full() {
        let mut buf = RingBuffer::new(4);
        let written = buf.write(&[1, 2, 3, 4, 5, 6]);
        assert_eq!(written, 4); // only 4 fit
        assert!(buf.is_full());
        assert_eq!(buf.available_space(), 0);

        let data = buf.read_all();
        assert_eq!(data, vec![1, 2, 3, 4]);
    }

    #[test]
    fn test_read_all_empty() {
        let mut buf = RingBuffer::new(8);
        let data = buf.read_all();
        assert!(data.is_empty());
    }

    #[test]
    fn test_multiple_write_read_cycles() {
        let mut buf = RingBuffer::new(8);

        buf.write(&[1, 2, 3]);
        assert_eq!(buf.read_all(), vec![1, 2, 3]);

        buf.write(&[4, 5]);
        assert_eq!(buf.read_all(), vec![4, 5]);

        buf.write(&[6, 7, 8, 9, 10, 11, 12, 13]);
        assert_eq!(buf.len(), 8);
        assert_eq!(buf.read_all(), vec![6, 7, 8, 9, 10, 11, 12, 13]);
    }

    #[test]
    fn test_default_capacity() {
        let buf = RingBuffer::default();
        assert_eq!(buf.capacity, 64 * 1024);
        assert!(buf.is_empty());
    }
}
