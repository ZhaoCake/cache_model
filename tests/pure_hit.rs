use cache_model::cache::{AccessRequest, AccessStatus, AccessType, Cache, CacheConfig};

#[test]
fn read_hit_returns_data() {
	// TODO: 补齐 Cache::new / Cache::access / CacheLine::fill 等实现后再开启此测试
	let _ = (
		AccessRequest {
			addr: 0,
			access_type: AccessType::Read,
			rsize: 2,
			wdata: 0,
			wmask: 0,
		},
		CacheConfig {
			addr_width: 8,
			cache_size: 32,
			line_size: 4,
		},
	);
	let _ = Cache::new;
	let _ = AccessStatus::Hit;
	todo!("enable this test after minimal hit-path implementation");
}
