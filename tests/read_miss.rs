use cache_model::cache::{AccessRequest, AccessStatus, AccessType, Cache, CacheConfig};

#[test]
fn read_miss_refill_then_hit() {
    let config = CacheConfig {
        addr_width: 16,
        cache_size: 64,
        line_size: 16,
    };
    let mut cache = Cache::new(config);
    // 模拟 memory 中的数据
    cache.memory.write_line(0, &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16][..]);
    
    // 第一次访问，应该是 miss
    let request = AccessRequest {
        access_type: AccessType::Read,
        addr: 0,
        rsize: 4,
        wdata: 0,
        wmask: 0,
    };
    let response = cache.access(request);
    assert_eq!(response.status, AccessStatus::Miss);
    assert!(!response.ready); // miss 时还未准备好数据
}

#[test]
fn read_miss_refill_then_hit_after_refill() {
    let config = CacheConfig {
        addr_width: 16,
        cache_size: 64,
        line_size: 16,
    };
    let mut cache = Cache::new(config);
    cache
        .memory
        .write_line(0, &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16][..]);

    // 第一次访问：预期 miss
    let req = AccessRequest {
        access_type: AccessType::Read,
        addr: 0,
        rsize: 4,
        wdata: 0,
        wmask: 0,
    };
    let first = cache.access(req);
    assert_eq!(first.status, AccessStatus::Miss);
    assert!(!first.ready);

    // refill 完成后再次访问：预期 hit + 正确数据
    let second = cache.access(req);
    assert_eq!(second.status, AccessStatus::Hit);
    assert_eq!(second.rdata, 0x04030201);
    assert!(second.ready);
}

#[test]
fn read_miss_populates_line() {
    let config = CacheConfig {
        addr_width: 16,
        cache_size: 64,
        line_size: 16,
    };
    let mut cache = Cache::new(config);
    cache
        .memory
        .write_line(0, &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16][..]);

    let req = AccessRequest {
        access_type: AccessType::Read,
        addr: 0,
        rsize: 4,
        wdata: 0,
        wmask: 0,
    };

    let _ = cache.access(req);

    // 预期：refill 后 line0 被正确填充
    assert!(cache.lines[0].valid);
    assert_eq!(cache.lines[0].tag, 0);
    assert_eq!(cache.lines[0].data, vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16]);

}