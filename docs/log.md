# 实现日志

## TODO（总体）

- [x] V1：命中通路（已完成）
	- [x] 地址拆分（tag/index/offset）
	- [x] CacheLine 读写/掩码/dirty
	- [x] Cache::access 命中处理
	- [x] hit 通路测试（4 条）
- [x] V1：读未命中（read miss + refill）
	- [x] 定义最小 miss 状态机接口/状态
	- [x] MockMemory（只支持 line read）
	- [x] 未命中：发起 line_read -> 装填 -> 重新完成请求
	- [x] 测试：冷启动读 miss、二次读 hit
- [ ] V1：写未命中（write-allocate）
	- [ ] 写 miss 触发 line_read
	- [ ] 装填后合并写入并置 dirty
	- [ ] 测试：写 miss 后读回
- [ ] V1：脏块回写（writeback）
	- [ ] victim 脏块回写到 MockMemory
	- [ ] 测试：替换触发回写
- [ ] V1：ready/stall 行为
	- [ ] miss 期间 ready=0
	- [ ] miss 完成后 ready=1
	- [ ] 测试：ready 门控

---

4/6

已经完成上述内容，下一步？
做，读未命中的通路。
未命中的通路就需要一个获取内存的方案了。
这里就采用一个 MockMemory 来模拟内存，先实现最简单的 line read 功能就行了。

 A. 命中通路（已完成）
1) read_hit_returns_data
	- 前置：line0.valid=1，tag=0，data=[1..16]
	- 输入：addr=0, rsize=4
	- 期望：Hit，rdata=0x04030201，ready=true

2) read_hit_with_different_offset
	- 前置：line0.valid=1，tag=0，data=[1..16]
	- 输入：addr=4 (offset=4), rsize=4
	- 期望：Hit，rdata=0x08070605，ready=true

3) write_hit_updates_data
	- 前置：line0.valid=1，tag=0，data=[1..16]
	- 输入：addr=0, wdata=0xFFFFFFFF, wmask=0xF
	- 期望：Hit，ready=true；随后读 addr=0 返回 0xFFFFFFFF

4) write_hit_respects_wmask
	- 前置：line0.valid=1，tag=0，data=[1..16]
	- 输入：addr=0, wdata=0xFFFFFFFF, wmask=0x5(0101)
	- 期望：Hit，ready=true；随后读 addr=0 返回 0x04FF02FF

B. 读未命中（准备实现）
5) read_miss_refill_then_hit
	- 前置：cache 全部 valid=0；MockMemory 在 line_addr=0 处存 [1..16]
	- 输入：addr=0, rsize=4
	- 期望：第一次返回 Miss / ready=false；完成 refill 后再次访问 addr=0 返回 Hit 且 rdata=0x04030201

6) read_miss_populates_line
	- 前置：cache 全部 valid=0；MockMemory 在 line_addr=0 处存 [1..16]
	- 输入：addr=0, rsize=4
	- 期望：refill 后 line0.valid=1，tag=0，data=[1..16]


---

4/3

完成guide。

然后先来做hit路径的。

---

先只做 hit 通路

暂时不实现 miss 状态机
暂时不接下层内存
先假设某一行已经预装好数据
验证 tag 比较、命中判定、读数据返回、写命中更新

再做 read miss

加最小 refill 逻辑
先不考虑 dirty 回写
先把“未命中 -> 从内存读整行 -> 填入 cache -> 完成请求”走通

最后做 write miss + writeback

写分配
dirty 替换回写
ready/stall 配合状态机

---

你第一阶段最简测试建议只测这 4 条：

第一组：纯 hit 通路

read_hit_returns_data
预先放入一行 valid=1
tag 匹配
读请求返回正确 rdata
hit=true

read_hit_with_different_offset
如果以后 line > 4B 用这个
你现在 line=4B，这条可以先不写

write_hit_updates_data
命中写入
data 被更新
dirty=1

write_hit_respects_wmask
例如只改低 1 字节
验证部分写逻辑

---

实现上建议再拆一层
先不要一上来写完整 Cache::access()，先拆几个小函数：

decode_address(addr) -> { tag, index, offset }
check_hit(line, tag) -> bool
read_word(line, offset, rsize) -> u32
write_word(line, offset, wdata, wmask)
这样你可以先给这些纯函数写单元测试，再拼总流程。


