use cache_model::cache::{AccessRequest, AccessStatus, AccessType, Cache, CacheConfig};

// 辅助函数：创建默认配置（64B cache, 16B line, 4 lines）
fn make_config() -> CacheConfig {
    CacheConfig {
        addr_width: 16,
        cache_size: 64,
        line_size: 16,
    }
}

// 辅助函数：构造写请求
fn write_req(addr: u32, wdata: u32, wmask: u8) -> AccessRequest {
    AccessRequest {
        access_type: AccessType::Write,
        addr,
        rsize: 0,
        wdata,
        wmask,
    }
}

// 辅助函数：构造读请求
fn read_req(addr: u32, rsize: u8) -> AccessRequest {
    AccessRequest {
        access_type: AccessType::Read,
        addr,
        rsize,
        wdata: 0,
        wmask: 0,
    }
}

/// 测试1：写 miss 替换脏块时触发回写
/// 场景：
///   - 先写 addr=0（tag=0, index=0），触发写分配，line 变脏
///   - 再写 addr=64（tag=1, index=0），替换同一 index 的脏块
///   - 检查 MockMemory 中 addr=0 处的数据已被更新（回写生效）
#[test]
fn write_miss_evicts_dirty_block_and_writeback() {
    let mut cache = Cache::new(make_config());

    // 在 memory 中 addr=0 的行预填数据
    cache.memory.write_line(0, &[0xA0; 16]);
    // 在 memory 中 addr=64 的行预填数据（同 index=0，不同 tag=1）
    cache.memory.write_line(64, &[0xB0; 16]);

    // Step 1: 写 addr=0 → 写 miss → 分配 line(index=0) 并写入，dirty=true
    let resp = cache.access(write_req(0, 0xDEAD_BEEF, 0xF));
    assert_eq!(resp.status, AccessStatus::Miss);

    // 确认 line 已经变脏
    assert!(cache.lines[0].dirty);
    assert!(cache.lines[0].valid);

    // Step 2: 写 addr=64 → 写 miss → 需要替换 index=0 的脏块
    //         writeback_if_dirty 应该先把脏数据写回 memory 的 addr=0 行
    let resp = cache.access(write_req(64, 0x1234_5678, 0xF));
    assert_eq!(resp.status, AccessStatus::Miss);

    // 验证：memory 中 addr=0 的行应该已经被更新（0xDEADBEEF 被回写）
    let mem_data = cache.memory.read_line(0);
    // 小端序：0xDEADBEEF → EF BE AD DE
    assert_eq!(mem_data[0], 0xEF);
    assert_eq!(mem_data[1], 0xBE);
    assert_eq!(mem_data[2], 0xAD);
    assert_eq!(mem_data[3], 0xDE);
    // 后续字节保持原来的 0xA0
    assert_eq!(mem_data[4], 0xA0);
}

/// 测试2：读 miss 替换脏块时触发回写
/// 场景：
///   - 先写 addr=16（tag=0, index=1），触发写分配，line 变脏
///   - 再读 addr=80（tag=1, index=1），替换同一 index 的脏块
///   - 检查 MockMemory 中 addr=16 处的数据已被更新
#[test]
fn read_miss_evicts_dirty_block_and_writeback() {
    let mut cache = Cache::new(make_config());

    // 在 memory 中 addr=16 的行预填数据
    cache.memory.write_line(16, &[0x11; 16]);
    // 在 memory 中 addr=80 的行预填数据（同 index=1，不同 tag=1）
    cache.memory.write_line(80, &[0x22; 16]);

    // Step 1: 写 addr=16 → 写 miss → 分配 line(index=1) 并写入，dirty=true
    let resp = cache.access(write_req(16, 0xCAFE_F00D, 0xF));
    assert_eq!(resp.status, AccessStatus::Miss);
    assert!(cache.lines[1].dirty);

    // Step 2: 读 addr=80 → 读 miss → 需要替换 index=1 的脏块
    //         writeback_if_dirty 应该先把脏数据写回 memory 的 addr=16 行
    let resp = cache.access(read_req(80, 4));
    assert_eq!(resp.status, AccessStatus::Miss);

    // 验证：memory 中 addr=16 的行应该已经被更新（0xCAFEF00D 被回写）
    let mem_data = cache.memory.read_line(16);
    // 小端序：0xCAFEF00D → 0D F0 FE CA
    assert_eq!(mem_data[0], 0x0D);
    assert_eq!(mem_data[1], 0xF0);
    assert_eq!(mem_data[2], 0xFE);
    assert_eq!(mem_data[3], 0xCA);
}

/// 测试3：替换干净块时不触发回写
/// 场景：
///   - 先读 addr=0，触发读 miss，refill 后 line 是干净的（dirty=false）
///   - 再写 addr=64（同 index=0，替换干净块），不应触发回写
///   - 验证 memory 中 addr=0 的数据保持不变
#[test]
fn evicting_clean_block_does_not_writeback() {
    let mut cache = Cache::new(make_config());

    // 在 memory 中预填数据
    cache.memory.write_line(0, &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16]);
    cache.memory.write_line(64, &[0xBB; 16]);

    // Step 1: 读 addr=0 → 读 miss → refill，line 是干净的
    let resp = cache.access(read_req(0, 4));
    assert_eq!(resp.status, AccessStatus::Miss);
    assert!(!cache.lines[0].dirty); // 读分配后 dirty 应为 false

    // Step 2: 写 addr=64 → 写 miss → 替换 index=0（干净块，不应回写）
    let resp = cache.access(write_req(64, 0, 0xF));
    assert_eq!(resp.status, AccessStatus::Miss);

    // 验证：memory 中 addr=0 的行应该保持原样（没有被回写覆盖）
    let mem_data = cache.memory.read_line(0);
    assert_eq!(mem_data, [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16]);
}

/// 测试4：回写后 dirty 标志被清除
/// 场景：
///   - 写 addr=0 使 line 变脏
///   - 写 addr=64 触发替换，回写脏块
///   - 验证替换后新 line 的 dirty 标志位正确
#[test]
fn writeback_clears_dirty_flag() {
    let mut cache = Cache::new(make_config());

    cache.memory.write_line(0, &[0; 16]);
    cache.memory.write_line(64, &[0; 16]);

    // Step 1: 写 addr=0 → 脏块
    cache.access(write_req(0, 0xFFFF_FFFF, 0xF));
    assert!(cache.lines[0].dirty);

    // Step 2: 写 addr=64 → 替换，回写后 fill 新行
    cache.access(write_req(64, 0xAAAA_AAAA, 0xF));

    // fill 会清 dirty，但 write_u32 又会置 dirty
    // 所以替换后的 line 仍然是 dirty 的（因为刚写入了新数据）
    assert!(cache.lines[0].dirty);
    assert_eq!(cache.lines[0].tag, 1); // addr=64 的 tag=1

    // 但 memory 中 addr=0 的数据应该已经被回写了
    let mem_data = cache.memory.read_line(0);
    assert_eq!(u32::from_le_bytes(mem_data[0..4].try_into().unwrap()), 0xFFFF_FFFF);
}

/// 测试5：替换无效行（valid=false）不触发回写
/// 场景：
///   - cache 初始状态所有行 valid=false
///   - 首次写 miss，victim 行无效，不应回写
///   - 程序不应 panic
#[test]
fn evicting_invalid_line_does_not_writeback() {
    let mut cache = Cache::new(make_config());

    cache.memory.write_line(0, &[0; 16]);

    // 直接写 miss，victim 行 valid=false，不会回写，不应 panic
    let resp = cache.access(write_req(0, 0x1234_5678, 0xF));
    assert_eq!(resp.status, AccessStatus::Miss);

    // 之后的读命中应该返回刚写入的数据
    let resp = cache.access(read_req(0, 4));
    assert_eq!(resp.status, AccessStatus::Hit);
    assert_eq!(resp.rdata, 0x1234_5678);
}

/// 测试6：多轮写-替换-回写，验证数据在 cache 和 memory 间正确流转
/// 场景：
///   - 写 addr=0 → 脏块 A
///   - 写 addr=64 → 替换 A，回写 A 到 memory，创建脏块 B
///   - 读 addr=0  → 替换 B，回写 B 到 memory，从 memory 读回 A
///   - 验证数据的完整性
#[test]
fn multiple_evictions_preserve_data() {
    let mut cache = Cache::new(make_config());

    cache.memory.write_line(0, &[0; 16]);
    cache.memory.write_line(64, &[0; 16]);

    // Round 1: 写 addr=0，数据 0x1111_1111
    cache.access(write_req(0, 0x1111_1111, 0xF));
    assert!(cache.lines[0].dirty);

    // Round 2: 写 addr=64，触发替换，回写 0x11111111 到 memory[0]
    cache.access(write_req(64, 0x2222_2222, 0xF));
    let mem_0 = cache.memory.read_line(0);
    assert_eq!(u32::from_le_bytes(mem_0[0..4].try_into().unwrap()), 0x1111_1111);

    // Round 3: 读 addr=0，触发替换，回写 0x22222222 到 memory[64]，从 memory 读回 0x11111111
    cache.access(read_req(0, 4));
    // 先验证 memory[64] 被回写了
    let mem_64 = cache.memory.read_line(64);
    assert_eq!(u32::from_le_bytes(mem_64[0..4].try_into().unwrap()), 0x2222_2222);

    // 再验证 cache 中 addr=0 可以命中并返回正确数据
    let resp = cache.access(read_req(0, 4));
    assert_eq!(resp.status, AccessStatus::Hit);
    assert_eq!(resp.rdata, 0x1111_1111);
}
