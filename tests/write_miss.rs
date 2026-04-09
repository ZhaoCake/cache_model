use cache_model::cache::{AccessRequest, AccessStatus, AccessType, Cache, CacheConfig};

#[test]
fn write_miss_allocate_then_read_back() {
    let config = CacheConfig {
        addr_width: 16,
        cache_size: 64,
        line_size: 16,
    };
    let mut cache = Cache::new(config);

    cache
        .memory
        .write_line(0, &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16][..]);

    let write_req = AccessRequest {
        access_type: AccessType::Write,
        addr: 0,
        rsize: 0,
        wdata: 0xFFFF_FFFF,
        wmask: 0xF,
    };
    let write_resp = cache.access(write_req);
    assert_eq!(write_resp.status, AccessStatus::Miss);
    assert!(!write_resp.ready);

    let read_req = AccessRequest {
        access_type: AccessType::Read,
        addr: 0,
        rsize: 4,
        wdata: 0,
        wmask: 0,
    };
    let read_resp = cache.access(read_req);
    assert_eq!(read_resp.status, AccessStatus::Hit);
    assert_eq!(read_resp.rdata, 0xFFFF_FFFF);
    assert!(read_resp.ready);
}

#[test]
fn write_miss_sets_dirty_and_tag() {
    let config = CacheConfig {
        addr_width: 16,
        cache_size: 64,
        line_size: 16,
    };
    let mut cache = Cache::new(config);

    cache
        .memory
        .write_line(0, &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16][..]);

    let write_req = AccessRequest {
        access_type: AccessType::Write,
        addr: 0,
        rsize: 0,
        wdata: 0x1234_5678,
        wmask: 0xF,
    };
    let write_resp = cache.access(write_req);
    assert_eq!(write_resp.status, AccessStatus::Miss);

    assert!(cache.lines[0].valid);
    assert!(cache.lines[0].dirty);
    assert_eq!(cache.lines[0].tag, 0);
}

#[test]
fn write_miss_respects_wmask_after_allocate() {
    let config = CacheConfig {
        addr_width: 16,
        cache_size: 64,
        line_size: 16,
    };
    let mut cache = Cache::new(config);

    cache
        .memory
        .write_line(0, &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16][..]);

    let write_req = AccessRequest {
        access_type: AccessType::Write,
        addr: 0,
        rsize: 0,
        wdata: 0xFFFF_FFFF,
        wmask: 0x5,
    };
    let write_resp = cache.access(write_req);
    assert_eq!(write_resp.status, AccessStatus::Miss);

    let read_req = AccessRequest {
        access_type: AccessType::Read,
        addr: 0,
        rsize: 4,
        wdata: 0,
        wmask: 0,
    };
    let read_resp = cache.access(read_req);
    assert_eq!(read_resp.status, AccessStatus::Hit);
    assert_eq!(read_resp.rdata, 0x04FF_02FF);
}
