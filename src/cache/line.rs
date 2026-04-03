#[derive(Debug, Clone)]
pub struct CacheLine {
    pub valid: bool,
    pub dirty: bool,
    pub tag: u32,
    pub data: LineData,
}

pub type LineData = Vec<u8>;

impl CacheLine {
    pub fn new(line_size: usize) -> Self {
        // TODO: 初始化 CacheLine
        let _ = line_size;
        todo!("CacheLine::new not implemented");
    }

    pub fn is_hit(&self, tag: u32) -> bool {
        // TODO: 命中判断逻辑
        let _ = tag;
        todo!("CacheLine::is_hit not implemented");
    }

    pub fn read_u32(&self, offset: usize, rsize: u8) -> u32 {
        // TODO: 按小端读取数据
        let _ = (offset, rsize);
        todo!("CacheLine::read_u32 not implemented");
    }

    pub fn write_u32(&mut self, offset: usize, wdata: u32, wmask: u8) {
        // TODO: 按 wmask 写入并设置 dirty
        let _ = (offset, wdata, wmask);
        todo!("CacheLine::write_u32 not implemented");
    }

    pub fn fill(&mut self, tag: u32, data: &[u8]) {
        // TODO: 填充整行数据
        let _ = (tag, data);
        todo!("CacheLine::fill not implemented");
    }
}
