use std::sync::Mutex;

pub struct BufferPool {
    byte_buffers: Mutex<Vec<Vec<u8>>>,
    string_buffers: Mutex<Vec<String>>,
    max_pool_size: usize,
}

impl BufferPool {
    pub fn new(max_pool_size: usize) -> Self {
        Self {
            byte_buffers: Mutex::new(Vec::with_capacity(max_pool_size)),
            string_buffers: Mutex::new(Vec::with_capacity(max_pool_size)),
            max_pool_size,
        }
    }

    pub fn acquire_byte_buffer(&self, capacity: usize) -> Vec<u8> {
        let mut pool = self.byte_buffers.lock().unwrap();
        if let Some(mut buf) = pool.pop() {
            buf.clear();
            if buf.capacity() < capacity {
                buf.reserve(capacity - buf.capacity());
            }
            buf
        } else {
            Vec::with_capacity(capacity)
        }
    }

    pub fn release_byte_buffer(&self, mut buf: Vec<u8>) {
        if buf.capacity() > 10 * 1024 * 1024 {
            return; // Drop huge buffers to prevent memory leak
        }
        let mut pool = self.byte_buffers.lock().unwrap();
        if pool.len() < self.max_pool_size {
            buf.clear();
            pool.push(buf);
        }
    }

    pub fn acquire_string_buffer(&self, capacity: usize) -> String {
        let mut pool = self.string_buffers.lock().unwrap();
        if let Some(mut s) = pool.pop() {
            s.clear();
            if s.capacity() < capacity {
                s.reserve(capacity - s.capacity());
            }
            s
        } else {
            String::with_capacity(capacity)
        }
    }

    pub fn release_string_buffer(&self, mut s: String) {
        if s.capacity() > 10 * 1024 * 1024 {
            return;
        }
        let mut pool = self.string_buffers.lock().unwrap();
        if pool.len() < self.max_pool_size {
            s.clear();
            pool.push(s);
        }
    }
}

impl Default for BufferPool {
    fn default() -> Self {
        Self::new(16)
    }
}
