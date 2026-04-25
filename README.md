# AST-Based Code RAG System（Rust + git2 限制版）详细设计文档
> 直接支持主流 Top20 编程语言 · 纯 git2 驱动 · 多语言语义级检索

---

## 0. 约束说明（非常重要）

本方案**严格遵守以下限制**：

- ❗只允许使用 `git2` crate 访问仓库
- ❗禁止使用 git CLI（如 `git diff` / `git log`）
- ❗禁止 shell 依赖（不调用外部进程）
- ❗所有版本管理、diff、文件扫描均基于 git2 实现
- ❗不依赖任何外部 Git 工具或 hooks

**为什么选择 git2？**
- 提供 Rust 原生 Git 实现，无需系统安装 Git
- 可精确控制内存和 IO，适合服务端嵌入
- 可获取任意版本的文件内容、树、diff，实现完全自托管
- 避免 shell 转义、进程管理带来的安全与性能问题

---

## 1. 项目目标

构建一个具备以下能力的系统：

- 基于 AST 的语义级代码切片（优于盲目文本切分）
- **直接支持主流 Top20 编程语言**，并具备同等语义解析精度
- 高质量 RAG 检索（支持函数、类型、模块、类等跨语言语义匹配）
- **完全基于 git2 的增量索引**（commit 级变更感知）
- 支持大规模仓库（单仓库 10 万+ LOC）高效全量与增量处理
- 达到生产级查询延迟（P99 < 200ms）
- 提供可扩展的语言注册框架，新增语言无需改动核心逻辑
- 健壮的错误处理与降级策略

量化目标：
- 全量索引时间：< 2 分钟（10 万 LOC，单语言仓库）
- 增量更新时长：< 10 秒（单次 commit 常规改动）
- 向量检索精度：Top1 ≥ 70%，Top5 ≥ 90%
- 资源占用：内存峰值 < 2GB，磁盘嵌入向量存储大小可控

---

## 2. 系统架构（git2 驱动）

```
       ┌──────────────┐
       │  Git Repo    │
       └──────┬───────┘
              │
    ┌─────────▼─────────┐
    │ git2 Repository   │  ← repository 模块
    └─────────┬─────────┘
              │
    ┌─────────▼─────────┐
    │ Tree Walker       │  ← repository 模块 + diff 模块
    └─────────┬─────────┘
              │
    ┌─────────▼─────────┐
    │ Multi-Language    │  ← parser 模块（Top20 语言支持）
    │ AST Parser        │
    └─────────┬─────────┘
              │
    ┌─────────▼─────────┐
    │ Analyzer          │  ← analyzer 模块（语言特定分析器）
    └─────────┬─────────┘
              │
    ┌─────────▼─────────┐
    │ Chunker           │  ← chunker 模块
    └─────────┬─────────┘
              │
    ┌─────────▼─────────┐
    │ Embedder          │  ← embedder 模块 (调用外部嵌入服务或本地模型)
    └─────────┬─────────┘
              │
    ┌─────────▼─────────┐
    │ Qdrant            │  ← storage 模块 (矢量存储)
    └───────────────────┘
              ▲
    ┌─────────┴─────────┐
    │ Query Engine      │  ← 查询接口、重排序、过滤
    └───────────────────┘
```

**数据流概要：**
1. `git2::Repository` 打开仓库，定位 HEAD 或指定 commit。
2. Tree Walker 遍历目标树，产出文件列表（`FileEntry`）。
3. 根据文件后缀或 shebang 选择对应的语言解析器（支持 Top20 语言）。
4. 语言特定 Parser 产出 CST/AST。
5. 语言特定 Analyzer 提取具名符号（函数、类、方法等）及其作用域。
6. Chunker 将符号定义连同文档注释、签名等切分为 `Chunk`，并计算内容哈希。
7. Embedder 把 `Chunk` 的文本表示转换为向量。
8. 全量索引：upsert 到 Qdrant collection。增量索引：对比新旧 commit diff，仅处理变更文件。
9. Query Engine 接收自然语言/代码片段，生成查询向量，从 Qdrant 检索，可选重排序。

**并发与流水线设计：**
- Tree walk 为单线程迭代，但产出文件可放入通道。
- Parser/Analyzer/Chunker 阶段采用多线程并行处理每个文件（使用 rayon）。
- Embedder 通过批量 API 调用，控制并发请求数，避免限流。
- 最终 upsert 批量提交。

---

## 3. 核心模块设计

### 3.1 repository 模块（git2 封装）

#### 职责
- 打开仓库（支持 bare 和 non-bare repos，支持 subdir 仓库）
- 读取指定 commit / tree / blob
- 提供 diff 能力（tree to tree, commit to commit）
- 缓存频繁访问的 commit 和 tree 对象以减少底层调用

#### 关键结构

```rust
pub struct GitRepo {
    repo: git2::Repository,
}

pub struct CommitInfo {
    pub oid: Oid,
    pub message: String,
    pub time: Time,
}

pub struct FileEntry {
    pub path: String,          // 相对于 repo root 的路径
    pub blob_id: Oid,
    pub file_mode: i32,        // 文件模式（普通文件、可执行等）
}
```

#### 核心能力详解

##### 1. 获取 HEAD commit
```rust
pub fn head_commit(&self) -> Result<CommitInfo>
```
- 通过 `repo.head()` 拿到引用，再 `peel_to_commit()` 解析。
- 同时支持 detached HEAD 状态，处理 reference 为非分支的情况。

##### 2. 遍历文件树
```rust
pub fn walk_tree(&self, tree: &Tree) -> Result<Vec<FileEntry>>
```
- 递归遍历 `git2::Tree`，跳过子模块和特殊类型。
- 只返回 blob 类型的条目。
- 可配置只索引特定扩展名的源码文件，以减少无效遍历。扩展名列表由语言注册表动态提供。

##### 3. 读取文件内容
```rust
pub fn read_blob(&self, oid: Oid) -> Result<Vec<u8>>
```
- 使用 `repo.find_blob(oid)` 后 `blob.content().to_vec()`。
- 对特别大的文件（ > 1MB），可跳过索引并记录日志，避免解析超时。

##### 4. 获取 commit 的 tree
```rust
pub fn commit_to_tree(&self, commit_oid: Oid) -> Result<Tree>
```
- 用于 diff 时的 old/new tree。

##### 5. 缓存设计
- 内部使用 LRU 缓存存储已解析的 `Commit` 和 `Tree` 对象，避免重复解析（`cached` crate 或自己实现）。
- 缓存 key 为 Oid，容量 100。

---

### 3.2 Diff 模块（完全替代 git diff）

#### 核心：使用 `git2::Diff`

```rust
pub fn diff_commits(&self, old_commit_oid: Oid, new_commit_oid: Oid) -> Result<DiffResult>
```

#### 输出结构

```rust
pub struct DiffResult {
    pub added: Vec<FileChange>,
    pub modified: Vec<FileChange>,
    pub deleted: Vec<String>,          // 被删除的文件路径
}

pub struct FileChange {
    pub path: String,
    pub old_blob_id: Option<Oid>,      // 对于新增文件为 None
    pub new_blob_id: Option<Oid>,      // 对于删除文件为 None
}
```

#### 实现关键点

```rust
let old_tree = self.repo.find_commit(old_commit_oid)?.tree()?;
let new_tree = self.repo.find_commit(new_commit_oid)?.tree()?;
let mut diff = self.repo.diff_tree_to_tree(
    Some(&old_tree),
    Some(&new_tree),
    Some(DiffOptions::new().ignore_filemode(true)),
)?;
```

遍历 diff deltas：识别 Added、Modified、Deleted、Renamed、Copied 状态，分别处理。对重命名/拷贝视为“删除旧路径 + 新增新路径”组合。

---

### 3.3 多语言 AST Parser 模块（Top20 语言支持）

#### 3.3.1 支持的语言列表（Top 20+）

根据 TIOBE / GitHub 活跃度，直接支持：

| # | 语言          | 标准扩展名           | tree-sitter 语法库          |
|---|---------------|----------------------|-----------------------------|
| 1 | Python        | .py, .pyw            | tree-sitter-python          |
| 2 | Java          | .java                | tree-sitter-java            |
| 3 | JavaScript    | .js, .jsx            | tree-sitter-javascript      |
| 4 | TypeScript    | .ts, .tsx            | tree-sitter-typescript      |
| 5 | C             | .c, .h               | tree-sitter-c               |
| 6 | C++           | .cpp, .cc, .hpp, .h  | tree-sitter-cpp             |
| 7 | C#            | .cs                  | tree-sitter-c-sharp         |
| 8 | Go            | .go                  | tree-sitter-go              |
| 9 | Rust          | .rs                  | tree-sitter-rust            |
| 10| Swift         | .swift               | tree-sitter-swift           |
| 11| Kotlin        | .kt, .kts            | tree-sitter-kotlin          |
| 12| PHP           | .php                 | tree-sitter-php             |
| 13| Ruby          | .rb                  | tree-sitter-ruby            |
| 14| Scala         | .scala               | tree-sitter-scala           |
| 15| Dart          | .dart                | tree-sitter-dart            |
| 16| Lua           | .lua                 | tree-sitter-lua             |
| 17| R             | .r, .R               | tree-sitter-r               |
| 18| Perl          | .pl, .pm             | tree-sitter-perl            |
| 19| Shell/Bash    | .sh, .bash           | tree-sitter-bash            |
| 20| SQL           | .sql                 | tree-sitter-sql             |
| +  | Markdown/文档 | .md                  | tree-sitter-markdown (可选) |

未来可扩展其他语言（如 Haskell, Elixir 等）。

#### 3.3.2 语言检测与解析器注册

使用 `LanguageRegistry` 统一管理所有语言解析器。

```rust
pub struct LanguageRegistry {
    languages: HashMap<String, LanguageConfig>,
    extension_map: HashMap<String, String>,   // ext -> lang_id
}

pub struct LanguageConfig {
    pub id: String,
    pub name: String,
    pub grammar: Language,                    // tree-sitter Language
    pub symbol_queries: HashMap<SymbolKind, String>, // 针对每种符号的 query
}
```

**自动语言检测策略：**
1. 根据文件扩展名查找 `extension_map`，若命中则直接确定语言。
2. 若无扩展名或映射表未覆盖，检查文件首行 shebang（例如 `#!/usr/bin/env python`）。
3. 若仍无法确定，则尝试所有语言解析，选择解析错误最少的语言（备选方案，开销较大，可关闭）。
4. 若完全未识别，按文本降级处理（标记语言为 `unknown`）。

#### 3.3.3 解析流程

```rust
pub fn parse_source(code: &[u8], language_id: &str) -> Result<Tree>
```

- 依据 `LanguageConfig` 中的 `grammar` 设置 `tree_sitter::Parser`。
- 解析获得 `Tree` （CST）。
- 如果解析完全失败（例如 `Tree` 含有 ERROR 节点但无法恢复），抛出 `ParserError::SyntaxError`。调用方将触发文本降级分块。

#### 3.3.4 语法加载

所有 tree-sitter 语法库通过 Cargo features 或条件编译静态链接。在编译时选择需要包含的语言，以减小二进制体积。或者采用动态加载（如 `libloading`），但为了简化，首选静态编译所有 Top20 语言。

#### 3.3.5 降级与容错

- 解析错误时，只丢弃不可恢复的文件（极少见），大部分情况下 tree-sitter 的错误恢复功能可提取部分符号。
- 回退机制：当解析器不可用（例如未编译某语言）时，使用基于行/段落切分的 `FallbackChunker`，生成 `kind: "text"` chunk。

---

### 3.4 Analyzer 模块（多语言符号提取）

#### 3.4.1 符号定义

```rust
pub enum SymbolKind {
    Function,
    Method,         // 类/结构体/接口内方法
    Class,
    Struct,
    Interface,
    Enum,
    Trait,
    Module,
    Namespace,
    TypeAlias,
    Variable,
    Constant,
    Macro,
    Other,
}
```

跨语言适用性：
- Java: `class`, `interface`, `method`（包括 static/instance）
- Python: `function`, `class`, `method`（类内函数）
- Rust: `function`, `struct`, `enum`, `trait`, `impl` 方法等
- C/C++: `function`, `struct`, `class`, `namespace`（C++）等
- Go: `function`, `struct`, `interface`
- ...（每种语言通过 query 映射到统一的 SymbolKind 集合）

#### 3.4.2 语言特定查询

每门语言配置一份 `tree-sitter` 查询文件，用来匹配各种符号定义节点。示例：

- **Python**:
    - 函数: `(function_definition ...) @function`
    - 类: `(class_definition ...) @class`
    - 方法: 在类内部匹配函数，标记为 `method`
- **Rust**:
    - 函数: `(function_item ...) @function`
    - 结构体: `(struct_item ...) @struct`
    - 枚举: `(enum_item ...) @enum`
    - Trait: `(trait_item ...) @trait`
    - 模块: `(mod_item ...) @module`
    - 方法: `(impl_item ... (function_item ...))` 等

所有查询存储在 `LanguageConfig.symbol_queries` 中，在 Analyzer 初始化时编译为 `tree_sitter::Query`。

#### 3.4.3 文档注释提取

每门语言有不同注释风格：
- C/Java/Rust: `// ...` 或 `/* ... */` 或 `/// ...`, `/** ... */`
- Python: `# ...` 或 `"""..."""`
- 针对符号上方连续的特殊注释行，提取为文档。

实现：在 CST 中，查找符号节点的前一个相邻注释节点或连续注释节点序列，判断是否为文档注释（通常以 `///` 或 `/**` 或 `#` 开头），拼接为 doc 字符串。

#### 3.4.4 签名构造

根据符号类型，递归拼接关键信息：
- 函数/方法：名称 + 参数列表 + 返回类型（如果查询可获取）
- 类/结构体：名称 + 基类/接口列表（如果存在）
- 模块/命名空间：全路径

代码示例：

```
// Rust
fn foo(x: i32, y: &str) -> bool
```

```
// Java
public class MyServlet extends HttpServlet
```

#### 3.4.5 模块/作用域路径

为了跨语言统一，构建 `module_path` 表示符号的完整逻辑路径：
- Python: `package.module.Class.method`
- Java: `com.example.MyClass.method`
- Rust: `crate::module::function`
- C++: `namespace::Class::method`

实现策略：
- 基于文件路径和语言语义推断模块层级。简单规则：文件路径的目录部分映射为模块层级，类/命名空间包裹为作用域。不强制要求 100% 编译器精度，保证检索区分能力即可。
- 使用 `tree-sitter` 的父节点信息（如 `mod_item`, `namespace_definition`, `class_definition`）动态构建。

---

### 3.5 Chunker 模块

#### Chunk 定义（Universal）

```rust
pub struct Chunk {
    pub id: String,                 // UUID 或基于内容的哈希
    pub repo: String,
    pub branch: String,
    pub commit: String,             // 对应的 commit Oid
    pub language: String,           // 如 "rust", "python", "java"
    pub file: String,               // 相对于仓库根
    pub module: String,             // 跨语言模块路径
    pub symbol: String,             // 符号名
    pub kind: String,               // 使用 SymbolKind 的字符串表示
    pub signature: String,
    pub doc: Option<String>,
    pub code: String,               // 完整符号定义源码
    pub start_line: usize,
    pub end_line: usize,
    pub content_hash: String,       // 用于增量更新的哈希
}
```

#### Hash 策略（关键）

`content_hash = sha256(code + signature + file + module + language)`  
加入 language 字段确保相同源码在不同语言语义不同时不会碰撞。

#### 切分算法

1. 从 Analyzer 获取符号列表。
2. 对每个符号，切割出对应的源代码（通过 `span` 从原始 `code` 中截取）。
3. 构造 `Chunk`，生成 `content_hash`。
4. 若 `doc` 非空，可额外生成一个文档片段 chunk（`signature + doc`），增强检索。

---

### 3.6 Embedder 模块

与先前设计一致，根据 `language` 字段可优化 prompt 模板，例如加入 `{language}` 信息。

嵌入文本模板示例：
```
[{language}] {signature}
{doc}
{code}
```

---

### 3.7 Indexer（核心 orchestrator）

#### 配置增强

```rust
pub struct IndexConfig {
    pub repo_path: PathBuf,
    pub branch: String,
    pub tracked_languages: Vec<String>,      // 限定索引的语言，为空表示所有
    pub batch_size: usize,
    pub qdrant_url: String,
    pub collection_name: String,
}
```

全量/增量流程逻辑不变，但文件过滤时根据文件扩展名匹配语言，只有支持的语言才进行解析索引。

#### 增量索引中处理语言变化

- 文件重命名可能改变扩展名，从而改变语言。在 diff 中，旧文件视为删除，新文件视为新增，自然触发新语言的解析。
- 其他细节与原设计相同。

---

### 3.8 查询引擎（Query Engine）

支持按语言过滤：
```rust
pub struct SearchFilters {
    pub language: Option<String>,   // 限定语言
    pub file: Option<String>,
    pub kind: Option<SymbolKind>,
    // ...
}
```

Qdrant 过滤条件中加入 `language` 字段，加速搜索。

---

## 4. Qdrant 设计

Collection 增加 `language` 字段并索引。Payload 示例：
```json
{
  "repo": "xxx",
  "branch": "main",
  "language": "rust",
  "file": "src/a.rs",
  "symbol": "foo",
  "kind": "function",
  "commit": "abc123"
}
```

---

## 5. 数据一致性设计

与原设计相同，增量为基于 commit 的最终一致。

---

## 6. 性能设计

支持多语言后，解析均为 CPU 密集，并发粒度仍是文件级。性能目标保持。

---

## 7. 验收标准

- 支持 Top20 语言的函数/方法/类/接口级提取及检索。
- 各语言各有单元测试覆盖常见符号提取。
- 检索质量统计按语言分别评估（至少覆盖 Top5 语言）。

---

## 8. 错误处理

增加语言相关错误：
- 未知语言或不受支持扩展名：降级为文本分块，记录告警。
- 语言解析器内部错误（如 tree-sitter panic）：使用 `catch_unwind` 隔离，回退到文本模式。

---

## 9. 项目结构（扩展）

```
crates/
├── core/
│   ├── repo           (git2 封装：打开、遍历、读取、diff)
│   ├── parser         (多语言 tree-sitter 集成，LanguageRegistry)
│   ├── analyzer       (语言特定查询，符号提取，签名构建)
│   ├── chunker        (切片定义、哈希、构建)
│   ├── embedder       (嵌入客户端、批处理、重试)
│   └── query          (查询构造、过滤、重排序)
├── storage/
│   └── qdrant         (集合管理、upsert、搜索、删除)
├── indexer/
│   ├── full           (全量索引逻辑)
│   ├── incremental    (增量索引逻辑)
│   └── state          (last_indexed_commit 持久化，可用 SQLite 或简单文件)
├── languages/         (语言包：每种语言一个 module 包含 queries 和配置)
│   ├── rust.rs
│   ├── python.rs
│   ├── java.rs
│   └── ...
└── server (可选)
    └── api            (HTTP 接口：触发索引、查询、状态)
```

---

## 10. 开发步骤（严格顺序并细化）

### Step 1：git2 仓库读取与 Diff
（不变）

### Step 2：多语言 AST 解析框架
- 实现 `LanguageRegistry`，注册至少 Top5 语言（Rust, Python, JavaScript, Java, Go）。
- 实现基于扩展名的语言检测。
- 编写每种语言的符号查询并测试。

### Step 3：多语言 Analyzer
- 实现通用 `Symbol` 提取及 `module_path` 构造。
- 添加单元测试覆盖各语言关键符号。

### Step 4：Chunker 生成与哈希
（不变）

### Step 5：Qdrant 接入与全量索引
（增加 language payload）

### Step 6：增量更新
（不变）

### Step 7：查询接口与质量评估
- 对每种支持语言标注查询测试集，评估 Top1/Top5。

### Step 8：扩展至 Top20 语言
- 逐个添加语言语法和查询，确保编译通过。
- 回归测试。

---

## 11. 风险及缓解措施

（新增多语言风险）
- **语言解析器质量参差不齐**：部分 tree-sitter 语法（如 Scala, Perl）可能不成熟，导致符号提取不全。解决：允许降级文本分块，并持续跟踪语法库更新。
- **签名提取复杂度**：不同语言语法差异巨大，精确参数列表提取困难。采用最佳努力策略，提取名称和大概参数（通过查询匹配简单节点），优先保证检索可用性。

---

## 12. 成功标准与扩展

系统达到多语言语义级检索，直接支持 Top20 编程语言，增量更新纯 git2 驱动，零 CLI 依赖。后续可扩展为支持代码导图、调用关系分析、跨文件上下文等。

---

## 结论

本文档详细阐述了一个**完全基于 git2、直接支持 Top20 主流编程语言**的代码语义索引系统。通过 tree-sitter 多语言解析框架与统一的符号抽象，实现跨语言的函数、类、方法等语义级切分与检索。系统坚持无外部 Git 依赖，提供生产级性能与鲁棒性，具备构建下一代智能代码助手的核心能力。