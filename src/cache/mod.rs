mod addr;
mod line;

pub use addr::{decode_address, AddressParts};
pub use line::{CacheLine, LineData};

#[derive(Debug, Clone, Copy)]
pub struct CacheConfig {
    pub addr_width: u8,
    pub cache_size: usize,
    pub line_size: usize,
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
}

impl Cache {
    pub fn new(config: CacheConfig) -> Self {
        // TODO: 初始化 Cache
        let _ = config;
        todo!("Cache::new not implemented");
    }

    pub fn num_lines(&self) -> usize {
        // TODO: 返回行数
        todo!("Cache::num_lines not implemented");
    }

    pub fn access(&mut self, request: AccessRequest) -> AccessResponse {
        // TODO: 访问流程（先只做 hit 通路）
        let _ = request;
        todo!("Cache::access not implemented");
    }
}
