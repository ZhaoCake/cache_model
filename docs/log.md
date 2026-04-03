# 实现日志

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

---
