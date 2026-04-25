# CodeRAG 实现任务清单

## 项目概述
基于 AST 的代码语义索引系统，使用 git2 和 tree-sitter，支持 Top20 编程语言。

---

## 开发步骤

### Step 1: 项目初始化与 git2 仓库模块
- [x] 创建 Cargo workspace 结构
- [x] 创建 core/repo 模块 (git2 封装)
  - [x] GitRepo 结构体
  - [x] CommitInfo, FileEntry 结构体
  - [x] head_commit() 方法
  - [x] walk_tree() 方法
  - [x] read_blob() 方法
  - [x] commit_to_tree() 方法
  - [x] LRU 缓存实现

### Step 2: Diff 模块
- [x] DiffResult, FileChange 结构体
- [x] diff_commits() 方法实现
- [x] 增量 diff 逻辑

### Step 3: 多语言 AST 解析框架
- [x] 创建 core/parser 模块
- [x] 实现 LanguageRegistry
- [x] 注册 Top5 语言:
  - [x] Rust
  - [x] Python
  - [x] JavaScript/TypeScript
  - [x] Java
  - [x] Go
- [x] 基于扩展名的语言检测
- [x] 语言配置 (LanguageConfig)

### Step 4: Analyzer 模块
- [x] SymbolKind 枚举定义
- [x] Symbol 结构体
- [x] 通用符号提取
- [x] module_path 构造
- [x] 文档注释提取
- [x] 签名构造

### Step 5: Chunker 模块
- [x] Chunk 结构体
- [x] content_hash 计算 (SHA256)
- [x] 切分算法实现

### Step 6: Embedder 模块
- [x] 嵌入文本模板
- [x] 嵌入客户端 (支持 OpenAI/本地模型)
- [x] 批处理逻辑

### Step 7: Qdrant Storage 模块
- [x] Collection 管理
- [x] Upsert/Delete 操作
- [x] 搜索操作
- [x] language 字段过滤

### Step 8: Indexer 模块
- [x] IndexConfig 配置
- [x] 全量索引 (full)
- [x] 增量索引 (incremental)
- [x] 状态持久化 (state)

### Step 9: CLI 应用
- [x] index 子命令
- [x] search 子命令
- [x] info 子命令
- [x] languages 子命令

### Step 10: Query Engine
- [x] SearchFilters 结构体
- [x] 查询构造
- [x] 过滤与重排序

### Step 11: 扩展至 Top20 语言
- [x] C
- [x] C++
- [x] C#
- [x] Swift
- [x] Kotlin
- [x] PHP
- [x] Ruby
- [x] Scala
- [x] Dart
- [x] Lua
- [x] R
- [x] Perl
- [x] Shell/Bash
- [x] SQL

### Step 12: 单元测试
- [x] 各语言符号提取测试 (Top5: Rust, Python, JavaScript, Java, Go)
- [x] Diff 模块测试
- [x] Chunker 测试
- [x] Parser 语言检测测试
- [ ] 集成测试

### Step 13: 质量评估
- [ ] Top1/Top5 检索准确率评估
- [ ] 性能测试

---

## 验收标准
- [x] 支持 Top20 语言的函数/方法/类/接口级提取及检索
- [x] 各语言各有单元测试覆盖常见符号提取 (Top5 语言)
- [ ] 检索质量统计按语言分别评估（至少覆盖 Top5 语言）

---

## 进度记录

### Iteration 1
项目初始化、核心模块实现、索引器、CLI 完成。

### Iteration 2
修复编译错误、完善测试、修复符号提取逻辑。

### Iteration 3 (当前)
所有测试通过，语言提取功能完善。
