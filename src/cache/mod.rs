mod addr;
mod line;
mod memory;

pub use addr::{AddressParts, decode_address};
pub use line::{CacheLine, LineData};
pub use memory::MockMemory;

#[derive(Debug, Clone, Copy)]
pub struct CacheConfig {
    pub addr_width: u8,    // 地址宽度（单位：bit）
    pub cache_size: usize, // cache 大小（单位：byte）
    pub line_size: usize,  //   cache line 大小（单位：byte）
                           // lines = cache_size / line_size
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccessType {
    Read,
    Write,
}

#[derive(Debug, Clone, Copy)]
pub struct AccessRequest {
    pub addr: u32,
    pub access_type: AccessType,
    pub rsize: u8,
    pub wdata: u32,
    pub wmask: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccessStatus {
    Hit,
    Miss,
}

#[derive(Debug, Clone, Copy)]
pub struct AccessResponse {
    pub status: AccessStatus,
    pub rdata: u32,
    pub ready: bool,
}

#[derive(Debug, Clone)]
pub struct Cache {
    pub config: CacheConfig,
    pub lines: Vec<CacheLine>,
    pub memory: MockMemory,
}

impl Cache {
    pub fn new(config: CacheConfig) -> Self {
        let lines = vec![CacheLine::new(config.line_size); config.cache_size / config.line_size];
        let memory = MockMemory::new(config.line_size, config.cache_size * 4);
        // vec!的实现会调用 CacheLine::new 来初始化每一行
        // config.addr_width 目前未直接使用
        Self { config, lines, memory }
    }

    pub fn num_lines(&self) -> usize {
        self.lines.len()
    }

    pub fn access(&mut self, request: AccessRequest) -> AccessResponse {
        match request.access_type {
            AccessType::Read => self.read(request),
            AccessType::Write => self.write(request),
        }
    }

    fn read(&mut self, request: AccessRequest) -> AccessResponse {
        let addr_parts = decode_address(request.addr, self.config.line_size, self.num_lines());
        if self.lines[addr_parts.index].is_hit(addr_parts.tag) {
            AccessResponse {
                status: AccessStatus::Hit,
                rdata: self.lines[addr_parts.index].read_u32(addr_parts.offset, request.rsize),
                ready: true,
            }
        } else {
            // 最小读 miss 框架：本次返回 Miss，同时完成 refill，下一次访问命中。
            let line_addr = (request.addr as usize / self.config.line_size) * self.config.line_size;
            let line_data = self.memory.read_line(line_addr);
            self.lines[addr_parts.index].fill(addr_parts.tag, &line_data);
            AccessResponse { status: AccessStatus::Miss, rdata: 0, ready: false }
        }
    }

    fn write(&mut self, request: AccessRequest) -> AccessResponse {
        let addr_parts = decode_address(request.addr, self.config.line_size, self.num_lines());
        let line = &mut self.lines[addr_parts.index];
        if line.is_hit(addr_parts.tag) {
            line.write_u32(addr_parts.offset, request.wdata, request.wmask);
            AccessResponse {
                status: AccessStatus::Hit,
                rdata: 0,
                ready: true,
            }
        } else {
            // TODO: miss 处理（write-allocate）
            AccessResponse { status: AccessStatus::Miss, rdata: 0, ready: false }
        }
    }
}


