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

 Date: 13/01/2026 00:01:53
*/


-- ----------------------------
-- Table structure for web3_user_info
-- ----------------------------
DROP TABLE IF EXISTS "public"."web3_user_info";
CREATE TABLE "public"."web3_user_info" (
  "id" int8 NOT NULL DEFAULT nextval('user_web3_info_id_seq'::regclass),
  "user_id" int8 NOT NULL,
  "chain_id" int8 NOT NULL,
  "address" varchar(128) COLLATE "pg_catalog"."default" NOT NULL,
  "created_at" timestamptz(6) NOT NULL DEFAULT now(),
  "updated_at" timestamptz(6) NOT NULL DEFAULT now()
)
;

-- ----------------------------
-- Records of web3_user_info
-- ----------------------------
INSERT INTO "public"."web3_user_info" VALUES (7414486567504449537, 7414486567504449536, 137, '0xd2d6506637aa33a4efbcbcf6b559b86e5f9a28dc', '2026-01-07 10:02:32.348818+08', '2026-01-07 10:02:32.348827+08');

-- ----------------------------
-- Indexes structure for table web3_user_info
-- ----------------------------
CREATE INDEX "idx_web3_address" ON "public"."web3_user_info" USING btree (
  "address" COLLATE "pg_catalog"."default" "pg_catalog"."text_ops" ASC NULLS LAST
);

-- ----------------------------
-- Uniques structure for table web3_user_info
-- ----------------------------
ALTER TABLE "public"."web3_user_info" ADD CONSTRAINT "unique_user_chain" UNIQUE ("user_id", "chain_id");

-- ----------------------------
-- Primary Key structure for table web3_user_info
-- ----------------------------
ALTER TABLE "public"."web3_user_info" ADD CONSTRAINT "user_web3_info_pkey" PRIMARY KEY ("id");

-- ----------------------------
-- Foreign Keys structure for table web3_user_info
-- ----------------------------
ALTER TABLE "public"."web3_user_info" ADD CONSTRAINT "fk_user" FOREIGN KEY ("user_id") REFERENCES "public"."users" ("id") ON DELETE CASCADE ON UPDATE NO ACTION;
