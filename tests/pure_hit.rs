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
