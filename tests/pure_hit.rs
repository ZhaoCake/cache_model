use cache_model::cache::{AccessRequest, AccessStatus, AccessType, Cache, CacheConfig};

#[test]
fn read_hit_returns_data() {
    // TODO: 补齐 Cache::new / Cache::access / CacheLine::fill 等实现后再开启此测试
    let config = CacheConfig {
        addr_width: 16,
        cache_size: 64,
        line_size: 16,
    };
    let mut cache = Cache::new(config);
    // 预先填充一行数据
    cache.lines[0].fill(0, &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16][..]);
    let request = AccessRequest {
        access_type: AccessType::Read,
        addr: 0, // 地址 0 对应 cache line 0
        rsize: 4, // 读取 4 字节
        wdata: 0,
        wmask: 0,
    };
    let response = cache.access(request);
    assert_eq!(response.status, AccessStatus::Hit);
    assert_eq!(response.rdata, 0x04030201); // 小端字节序，最低地址存1，最高地址存4
    assert!(response.ready);    
}

#[test]
fn read_hit_with_different_offset() {
    let config = CacheConfig {
        addr_width: 16,
        cache_size: 64,
        line_size: 16,
    };
    let mut cache = Cache::new(config);
    cache.lines[0].fill(0, &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16][..]);
    let request = AccessRequest {
        access_type: AccessType::Read,
        addr: 4, // 地址 4 对应 cache line 0 的 offset 4
        rsize: 4,
        wdata: 0,
        wmask: 0,
    };
    let response = cache.access(request);
    assert_eq!(response.status, AccessStatus::Hit);
    assert_eq!(response.rdata, 0x08070605); // 从 offset 4 开始读取，依次是5、6、7、8
    assert!(response.ready);    
}

#[test]
fn write_hit_updates_data() {
    let config = CacheConfig {
        addr_width: 16,
        cache_size: 64,
        line_size: 16,
    };
    let mut cache = Cache::new(config);
    cache.lines[0].fill(0, &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16][..]);
    let request = AccessRequest {
        access_type: AccessType::Write,
        addr: 0,
        rsize: 0,
        wdata: 0xFFFFFFFF,
        wmask: 0xF, // 写入全部4字节
    };
    let response = cache.access(request);
    assert_eq!(response.status, AccessStatus::Hit);
    assert!(response.ready);
    // 验证数据已更新
    let read_request = AccessRequest {
        access_type: AccessType::Read,
        addr: 0,
        rsize: 4,
        wdata: 0,
        wmask: 0,
    };
    let read_response = cache.access(read_request);
    assert_eq!(read_response.status, AccessStatus::Hit);
    assert_eq!(read_response.rdata, 0xFFFFFFFF); // 数据应该被更新为全1
    assert!(read_response.ready);    
}

#[test]
fn write_hit_respects_wmask() {
    let config = CacheConfig {
        addr_width: 16,
        cache_size: 64,
        line_size: 16,
    };
    let mut cache = Cache::new(config);
    cache.lines[0].fill(0, &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16][..]);
    let request = AccessRequest {
        access_type: AccessType::Write,
        addr: 0,
        rsize: 0,
        wdata: 0xFFFFFFFF,
        wmask: 0x5, // 写入第0和第2字节 // 0101
    };
    let response = cache.access(request);
    assert_eq!(response.status, AccessStatus::Hit);
    assert!(response.ready);
    // 验证数据已更新
    let read_request = AccessRequest {
        access_type: AccessType::Read,
        addr: 0,
        rsize: 4,
        wdata: 0,
        wmask: 0,
    };
    let read_response = cache.access(read_request);
    assert_eq!(read_response.status, AccessStatus::Hit);
    assert_eq!(read_response.rdata, 0x04FF02FF); // 第0和第2字节被更新为0xFF，其他字节保持不变
    assert!(read_response.ready);    
}