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
            // 读 miss 路径：
            // 1) 先检查 victim（当前 index 对应行）是否脏，若脏则回写到 memory
            // 2) 从 memory 读取目标行数据并 fill 进 cache line
            // 3) 返回 Miss + ready=false（数据下个周期才可用）
            let needs_writeback = {
                let line = &self.lines[addr_parts.index];
                line.valid && line.dirty
            };
            if needs_writeback {
                self.writeback_if_dirty(addr_parts.index);
            }
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
            self.handle_write_miss(request, addr_parts)
        }
    }

    fn handle_write_miss(&mut self, request: AccessRequest, addr_parts: AddressParts) -> AccessResponse {
        // 写 miss 处理流程（write-allocate 策略）：
        // 1) 选择 victim：直接映射下，index 对应行就是唯一候选
        // 2) 若 victim 有效且脏，先回写（writeback_if_dirty）
        // 3) 从 memory 读取目标行数据并 fill 进 cache line
        // 4) 对 fill 后的行执行 write_u32（自动置 dirty）
        // 5) 返回 Miss + ready=false
        let needs_writeback = {
            let line = &self.lines[addr_parts.index];
            line.valid && line.dirty
        };
        if needs_writeback {
            self.writeback_if_dirty(addr_parts.index);
        }
        let line_addr = (request.addr as usize / self.config.line_size) * self.config.line_size;
        let line_data = self.memory.read_line(line_addr);
        let line = &mut self.lines[addr_parts.index];
        line.fill(addr_parts.tag, &line_data);
        line.write_u32(addr_parts.offset, request.wdata, request.wmask);

        AccessResponse {
            status: AccessStatus::Miss,
            rdata: 0,
            ready: false,
        }
    }

    fn writeback_if_dirty(&mut self, index: usize) {
        // 脏块回写：将 cache line 中被修改过的数据写回下一级存储
        // 条件：line 必须同时 valid 且 dirty，才需要回写
        // 步骤：
        //   1) 用 tag + index 还原该行在 memory 中的起始地址（line_addr）
        //      公式：line_addr = tag × (line_size × num_lines) + index × line_size
        //   2) 调用 memory.write_line 将整行数据写入
        //   3) 清除 dirty 标志（数据已与 memory 一致）
        if self.lines[index].valid && self.lines[index].dirty {
            let line_addr = (self.lines[index].tag as usize) * self.config.line_size * self.num_lines() + index * self.config.line_size;
            self.memory.write_line(line_addr, &self.lines[index].data);
            self.lines[index].dirty = false; // 回写后清除 dirty 位，数据已与 memory 一致
        }
    }
}


