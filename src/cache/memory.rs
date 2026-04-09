#[derive(Debug, Clone)]
pub struct MockMemory {
    pub line_size: usize,
    pub data: Vec<u8>,
}

impl MockMemory {
    pub fn new(line_size: usize, size: usize) -> Self {
        Self {
            line_size,
            data: vec![0; size],
        }
    }

    pub fn read_line(&self, line_addr: usize) -> Vec<u8> {
        if line_addr + self.line_size > self.data.len() {
            panic!("read_line out of bounds: line_addr={line_addr}, line_size={}", self.line_size);
        }
        self.data[line_addr..line_addr + self.line_size].to_vec()
    }

    pub fn write_line(&mut self, line_addr: usize, line_data: &[u8]) {
        if line_addr + self.line_size > self.data.len() {
            panic!("write_line out of bounds: line_addr={line_addr}, line_size={}", self.line_size);
        }
        if line_data.len() != self.line_size {
            panic!("write_line data size mismatch: expected {}, got {}", self.line_size, line_data.len());
        }
        self.data[line_addr..line_addr + self.line_size].copy_from_slice(line_data);
    }
}
