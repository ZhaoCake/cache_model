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
        // TODO: 按 line_addr 读取一整行
        let _ = line_addr;
        todo!("MockMemory::read_line not implemented");
    }

    pub fn write_line(&mut self, line_addr: usize, line_data: &[u8]) {
        // TODO: 按 line_addr 写回一整行
        let _ = (line_addr, line_data);
        todo!("MockMemory::write_line not implemented");
    }
}
