# CodeRAG 文档解析实现进度

## 测试结果
- ✅ 64 测试全过（58 core + 3 indexer + 2 storage + 1 doctest）
- ✅ 2 测试跳过（git 仓库依赖）
- ✅ 0 编译警告
- ✅ release build 成功

---

## 文档格式支持进度

### P0: 纯文本格式（✅ 已完成）

| 格式 | 扩展名 | 处理方式 | 测试状态 |
|------|--------|----------|----------|
| Markdown | .md, .markdown, .mdx | 去标记语法 | ✅ 通过 |
| Text | .txt, .text | 直接分块 | ✅ 通过 |
| XML | .xml | 去标签提取文本 | ✅ 通过 |
| JSON | .json, .jsonl, .ndjson | 保留结构化文本 | ✅ 通过 |
| TOML | .toml | 直接分块 | ✅ 通过 |
| INI | .ini, .cfg, .conf, .properties | 直接分块 | ✅ 通过 |
| CSV | .csv, .tsv | 保留表格文本 | ✅ 通过 |
| Log | .log | 逐行分块 | ✅ 通过 |

**特殊文件识别**: LICENSE, README, Makefile, Dockerfile 等无扩展名文件自动识别为文本格式。

### P1: 二进制文档格式（✅ 已完成，需 doc-p1 feature）

| 格式 | 扩展名 | 处理方式 | 实现状态 |
|------|--------|----------|----------|
| HTML | .html, .htm | 去标签 + 实体解析 | ✅ 通过 |
| DOCX | .docx | ZIP + XML (word/document.xml) | ✅ 依赖 zip crate |
| ODT | .odt | ZIP + XML (content.xml) | ✅ 依赖 zip crate |
| XLSX | .xlsx | ZIP + XML (sharedStrings + sheets) | ✅ 依赖 zip crate |
| XLS | .xls | BIFF 格式 | ⚠️ 预留 calamine 接口 |
| ODS | .ods | ZIP + XML (content.xml) | ✅ 依赖 zip crate |
| PDF | .pdf | 文本提取 | ⚠️ 预留 pdf-extract 接口 |

### P2: 压缩包格式（✅ 已完成，需 doc-p2 feature）

| 格式 | 扩展名 | 处理方式 | 实现状态 |
|------|--------|----------|----------|
| ZIP | .zip | 解压后递归处理 | ✅ 依赖 zip crate |
| TAR | .tar | 解压后递归处理 | ✅ 依赖 tar crate |
| GZ | .gz, .tgz | 解压为单文件 | ✅ 依赖 flate2 crate |

---

## Feature 配置

```toml
[dependencies]
# P0 默认启用
coderag-core = "0.1"  # 或 coderag-core = { version = "0.1", features = ["doc-p0"] }

# P1 二进制文档支持
coderag-core = { version = "0.1", features = ["doc-p1"] }

# P2 压缩包支持
coderag-core = { version = "0.1", features = ["doc-p2"] }

# 全部启用
coderag-core = { version = "0.1", features = ["doc-p0", "doc-p1", "doc-p2"] }
```

---

## 模块结构

```
crates/core/src/document/
├── mod.rs              # DocFormat 枚举 + 格式检测
├── text_chunker.rs     # P0 纯文本分块器 (13 个测试)
├── binary_extractor.rs # P1 二进制提取器 (4 个测试)
└── archive_handler.rs  # P2 压缩包处理器 (3 个测试)
```

## 集成点

- `FullIndexer::run()`: 优先 AST 提取，回退到文档解析
- `TextChunker`: 段落感知分块 + 重叠保持上下文
- 所有文档 chunk 使用 `kind: "document"` 标识

---

## 新增依赖

| Crate | 版本 | 用途 | Feature |
|-------|------|------|---------|
| zip | 2.0 | ZIP/OOXML 读取 | doc-p1, doc-p2 |
| flate2 | 1.0 | GZIP 解压 | doc-p2 |
| tar | 0.4 | TAR 解压 | doc-p2 |
