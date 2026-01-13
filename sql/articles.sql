/*
 Navicat Premium Data Transfer

 Source Server         : local
 Source Server Type    : PostgreSQL
 Source Server Version : 170007 (170007)
 Source Host           : localhost:5432
 Source Catalog        : blog_v2
 Source Schema         : public

 Target Server Type    : PostgreSQL
 Target Server Version : 170007 (170007)
 File Encoding         : 65001

 Date: 13/01/2026 00:01:30
*/


-- ----------------------------
-- Table structure for articles
-- ----------------------------
DROP TABLE IF EXISTS "public"."articles";
CREATE TABLE "public"."articles" (
  "id" int8 NOT NULL DEFAULT nextval('articles_id_seq'::regclass),
  "uid" int8 NOT NULL,
  "title" varchar(255) COLLATE "pg_catalog"."default" NOT NULL,
  "description" text COLLATE "pg_catalog"."default" NOT NULL DEFAULT ''::text,
  "content" text COLLATE "pg_catalog"."default" NOT NULL,
  "status" int4 NOT NULL DEFAULT 3,
  "likes" int8 NOT NULL DEFAULT 0,
  "views" int8 NOT NULL DEFAULT 0,
  "collects" int8 NOT NULL DEFAULT 0,
  "cover_urls" text[] COLLATE "pg_catalog"."default" NOT NULL DEFAULT '{}'::text[],
  "created_at" timestamptz(6) NOT NULL DEFAULT CURRENT_TIMESTAMP,
  "updated_at" timestamptz(6) NOT NULL DEFAULT CURRENT_TIMESTAMP,
  "deleted_at" timestamptz(6)
)
;

-- ----------------------------
-- Records of articles
-- ----------------------------
INSERT INTO "public"."articles" VALUES (7416398724169076736, 1, 'Test Article', 'Desc', 'Content', 3, 0, 0, 0, '{}', '2026-01-12 16:40:45.867923+08', '2026-01-12 16:40:45.867923+08', NULL);
INSERT INTO "public"."articles" VALUES (7416402011509362688, 1, 'Test Title', 'Test Description', 'Test Content', 3, 0, 0, 0, '{}', '2026-01-12 16:53:49.630433+08', '2026-01-12 16:53:49.630433+08', NULL);
INSERT INTO "public"."articles" VALUES (7416402102940995584, 7414486567504449536, 'Rust 微服务实战：构建高性能文章社区后端', '本文将探讨如何利用 Rust、Axum 和 SQLx 构建一个生产级别的文章管理微服务，并深入分析其内存安全与并发优势。', 'Content...', 3, 0, 0, 0, '{}', '2026-01-12 16:54:11.430203+08', '2026-01-12 16:54:11.430203+08', NULL);
INSERT INTO "public"."articles" VALUES (7416403873000198144, 7414486567504449536, 'Rust', '本文将探讨如何利用 Rust、Axum 和 SQLx 构建一个生产级别的文章管理微服务，并深入分析其内存安全与并发优势。', '# Rust 微服务实战

在当前的技术架构中，**高性能**和**类型安全**是后端开发的追求。本文将展示如何从零开始构建一个文章管理服务。

## 为什么选择 Rust？

1. **内存安全**：零成本抽象且没有 GC 压力。
2. **强类型枚举**：正如我们在代码中定义的 `ArticleStatus`，编译时即可排除非法状态。
3. **并发模型**：基于 Tokio 的异步运行时可以轻松处理万级并发。

## 核心代码示例

以下是我们定义的 `ArticleDetail` 核心结构体：

```rust
#[derive(Clone, FromRow, Serialize, Deserialize)]
pub struct ArticleDetail {
    pub id: i64,
    pub title: String,
    pub content: String,
    pub status: ArticleStatus,
    // ... 更多字段
}
```

## 数据库设计

我们采用 PostgreSQL 的 `TEXT[]` 数组来存储 `cover_urls`，并配合软删除（Soft Delete）逻辑：

- **优点**：数据可恢复，查询效率高。
- **索引优化**：通过 `idx_articles_status` 提升过滤速度。

> 总结：Rust 不仅仅是一门系统语言，它在 Web 微服务领域正展现出惊人的生产力。', 3, 0, 0, 0, '{https://cdn.example.com/images/rust-logo.png,https://cdn.example.com/images/microservices-arch.jpg}', '2026-01-12 17:01:13.444359+08', '2026-01-12 17:29:13.155148+08', '2026-01-12 17:30:44.514739+08');

-- ----------------------------
-- Indexes structure for table articles
-- ----------------------------
CREATE INDEX "idx_articles_deleted_at" ON "public"."articles" USING btree (
  "deleted_at" "pg_catalog"."timestamptz_ops" ASC NULLS LAST
);
CREATE INDEX "idx_articles_status" ON "public"."articles" USING btree (
  "status" "pg_catalog"."int4_ops" ASC NULLS LAST
) WHERE deleted_at IS NULL;
CREATE INDEX "idx_articles_uid" ON "public"."articles" USING btree (
  "uid" "pg_catalog"."int8_ops" ASC NULLS LAST
);

-- ----------------------------
-- Checks structure for table articles
-- ----------------------------
ALTER TABLE "public"."articles" ADD CONSTRAINT "status_check" CHECK (status = ANY (ARRAY[1, 2, 3]));

-- ----------------------------
-- Primary Key structure for table articles
-- ----------------------------
ALTER TABLE "public"."articles" ADD CONSTRAINT "articles_pkey" PRIMARY KEY ("id");
