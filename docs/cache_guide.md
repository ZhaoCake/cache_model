# Cache 实现方案

## 1 整体架构

当前阶段为第一阶段，实现一个直接映射的Cache模型，目的是实现基本结构，后续逐步完善参数化设计，增加全相联、组相联等功能。


## 2 接口定义

### 2.1 CPU 对 Cache 的 request接口

- addr: 8位地址输入（内部根据需要截断）
- op： 1位操作类型，0表示读，1表示写
- rsize： 2位读数据大小输入（00=1字节，01=2字节，10=4字节，仅读操作有效）
- wdata： 32位写数据输入（仅写操作有效，可单字节，wmask控制）
- wmask： 4位写掩码输入（仅写操作有效，每位对应一个字节，1表示写入该字节，0表示不写入）

> req_id: 8位请求ID输入，用于标识不同的请求，便于调试和跟踪（可选，所以暂时不做）

### 2.2 Cache 对 CPU 的 response接口

- hit: 1位命中标志输出，1表示命中，0表示未命中
- rdata: 32位读数据输出（仅读操作有效）
- stall 或 ready: 1位接口，表示Cache是否准备好接受下一个请求，1表示准备好，0表示需要等待

> - miss_type: compulsory, conflict, capacity miss的类型输出（可选，暂时不做）

### 2.3 Cache 对下层内存的接口

- line_read_req: 按cache line对齐读。
- line_read_resp: 返回整行数据。
- line_write_req: 脏块回写整行。
- mem_ready: 内存接口准备好信号。

## 3 Cache 参数设计

> 由于第一阶段直接做直接映射，因此不需要替换算法和相关参数设计，后续阶段再完善。

- 地址拆分：TAG + Index + Block Offset
- Cache line大小：4字节（32位），即Block Offset为2位（这样最快跑通）

ADDR_WIDTH = 8 （为了方便测试，地址宽度暂时设置为8位，后续增加参数化设计）
CACHE_SIZE = 32B (8 lines * 4B/line)
LINE_SIZE = 4B
ASSOCIATIVITY = 1（直接映射）
NUM_LINES = CACHE_SIZE / LINE_SIZE = 8
INDEX_BITS = log2(NUM_LINES) = 3
OFFSET_BITS = log2(LINE_SIZE) = 2
TAG_BITS = ADDR_WIDTH - INDEX_BITS - OFFSET_BITS

状态位：

valid
dirty
tag
data[4]

## 4 比较过程

在直接映射里：

1. 先由 Index 定位到唯一一行。
2. 比较该行的 Tag 和请求地址的 Tag。
3. 如果 Valid=1 且 Tag 相等，就是命中。
4. 只要 Tag 不同（或 Valid=0）就是未命中，不会去别的地方再找。
5. 未命中后是否“直接换”取决于该行状态：
6. 如果 Dirty=1，先回写旧行，再装填新行；如果 Dirty=0，可以直接装填覆盖。

## 5 写策略

- 写回（Write-back）：写操作先修改Cache中的数据块，标记为Dirty，只有当该块被替换出Cache时才写回内存。优点是减少内存写操作，提高性能；缺点是增加了数据一致性维护的复杂度。
- 写分配（Write-allocate）：写未命中时，先将对应的Cache line读入Cache，然后再进行写操作。适用于局部性较好的写操作，可以提高性能；缺点是增加了未命中时的延迟。

## 6* 时序语义

由于是模拟器实现，时序语义可以简化为：
1. CPU发出请求，Cache立即处理。
2. 如果命中，立即返回数据和hit信号。
3. 如果未命中，Cache开始处理miss，期间stall信号为1，表示需要等待。处理完成后返回数据和hit信号，stall信号恢复为0。

miss状态机：
idle->lookup->writeback->refill
并说明每个状态的行为和转移条件。

可执行的折中方案（推荐 V1）：

采用“请求在拍边界受理”的 tick 模型。
命中固定 1-cycle 返回。
未命中走状态机：Idle -> Lookup -> (WriteBack) -> Refill -> Respond。
miss 期间 ready=0，阻塞新请求。
内存侧延迟参数化：mem_read_latency、mem_write_latency（先固定常数）。

## 7 读写粒度与字节序规则

不应该支持非对齐访问。如果是4字节，则地址必须是4的倍数；如果是2字节，则地址必须是2的倍数；如果是1字节，则地址可以是任意值。

字节序，小端序，即低地址存储数据的低字节，高地址存储数据的高字节。

## 8 测试方案

试怎么设计（TDD + SDD）
先从“规格驱动的可观察行为”拆测试，不从内部实现拆。

测试分层：

1. 单元测试（纯函数）
2. 组件测试（Cache + MockMemory）
3. 场景测试（多步访问序列）
4. 回归测试（历史 bug 用例固化）

最小测试矩阵（建议先写这 12 条）：

1. 冷启动读未命中后装填成功
2. 同地址二次读命中
3. 读命中返回值正确（按小端拼接）
4. 写命中 + wmask 部分写正确
5. 写命中 dirty 置位
6. 写未命中（write-allocate）后写入正确
7. 脏行被替换时先回写再装填
8. 非脏行替换不回写
9. ready=0 时新请求不被接收
10. miss 结束后 ready 恢复
11. 非对齐访问被拒绝（或返回错误码，按你规范）
12. reset 后 valid 全 0、统计清零

断言重点（每条用例都尽量覆盖其中几项）：

1. 功能断言：hit/rdata
2. 协议断言：ready/stall 时机
3. 副作用断言：valid/dirty/tag/data 变化
4. 外部交互断言：MockMemory 收到的 read/writeback 次数和地址
5. 统计断言：hit/miss/writeback 计数

TDD 落地节奏（很实用）：

先写 3 条最小红测：冷启动读 miss、二次读 hit、写命中 dirty。

1. 实现最小状态机让其转绿。
2. 再补替换与回写相关红测。
3. 最后补异常与边界（非对齐、wmask=0、地址边界）。

SDD 文档建议补 3 段：

1. 时序契约：请求何时被接受、响应何时有效、ready/stall 极性。
2. 状态机契约：每个状态允许/禁止的输入与输出。
3. 测试验收表：每条需求对应至少 1 条测试编号（可追踪）。

## 9 注意事项

### 9.1 Cache 会 not ready的情况

1. 命中失败，处理miss
2. 回写脏块，装填新行
3. 做写分配：写未命中，先读入cache line，再修改目标字节，和1一起看。装填和合并写数据期间不接受新请求。
4. 单端口阵列被占用：Tag或 Data array只有一个访问端口，同一周期有其他请求。？需要例子。
5. 存在流水线冲突或结构冲突：上一个请求没完成，新的请求会引起同组冲突。比如上面一条。
6. 下层内存接口未就绪。memory bus忙，仲裁失败。
7. 启动刷新失效维护操作。

总的来说，就是未完成和装填合并的时候就不能接受新请求了。
