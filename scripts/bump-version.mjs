#!/usr/bin/env node

/**
 * 统一修改全项目版本号
 *
 * 用法: node scripts/bump-version.mjs <version>
 * 示例: node scripts/bump-version.mjs 0.2.0
 *
 * 修改范围:
 *   - package.json (root)
 *   - src-tauri/Cargo.toml
 *   - src-tauri/tauri.conf.json
 *   - site/src/i18n/translations.ts (hero.badge)
 *   - site/src/components/Download/Download.tsx (下载链接)
 *   - site/src/components/Hero/Hero.tsx (版本徽章)
 */

import { readFileSync, writeFileSync } from "fs";
import { resolve, dirname } from "path";
import { fileURLToPath } from "url";

const __dirname = dirname(fileURLToPath(import.meta.url));
const root = resolve(__dirname, "..");

const newVersion = process.argv[2];
if (!newVersion || !/^\d+\.\d+\.\d+(-[\w.]+)?$/.test(newVersion)) {
  console.error("用法: node scripts/bump-version.mjs <version>");
  console.error("示例: node scripts/bump-version.mjs 0.2.0");
  process.exit(1);
}

function filePath(rel) {
  return resolve(root, rel);
}

function escapeRegex(str) {
  return str.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}

function replaceInFile(rel, search, replace) {
  const abs = filePath(rel);
  let content = readFileSync(abs, "utf-8");
  const before = content;
  const matched = typeof search === "string" ? content.includes(search) : search.test(content);
  if (!matched) {
    console.warn(`  ⚠ 未匹配: ${rel}`);
    return false;
  }
  content = content.replace(search, replace);
  if (content !== before) {
    writeFileSync(abs, content, "utf-8");
  }
  return true;
}

function replaceJsonVersion(rel) {
  const abs = filePath(rel);
  const json = JSON.parse(readFileSync(abs, "utf-8"));
  const old = json.version;
  json.version = newVersion;
  writeFileSync(abs, JSON.stringify(json, null, 2) + "\n", "utf-8");
  console.log(`  ✓ ${rel}: ${old} → ${newVersion}`);
}

console.log(`\n🔧 统一修改版本号 → ${newVersion}\n`);

// 保存旧版本号（在 package.json 被覆盖前）
const oldVersion = JSON.parse(readFileSync(filePath("package.json"), "utf-8")).version;

// --- package.json ---
console.log("📦 package.json:");
replaceJsonVersion("package.json");

// --- Cargo.toml ---
console.log("\n🦀 Cargo.toml:");
replaceInFile("src-tauri/Cargo.toml", /^version = ".*"$/m, `version = "${newVersion}"`);
console.log(`  ✓ src-tauri/Cargo.toml → ${newVersion}`);

// --- tauri.conf.json ---
console.log("\n⚙️  tauri.conf.json:");
replaceInFile("src-tauri/tauri.conf.json", /"version": ".*"/, `"version": "${newVersion}"`);
console.log(`  ✓ src-tauri/tauri.conf.json → ${newVersion}`);

// --- Site: 同步 site 文件中的 vX.Y.Z ---
const oldTag = `v${oldVersion}`;
const newTag = `v${newVersion}`;

if (oldVersion !== newVersion) {
  console.log(`\n🌐 Site (同步 ${oldTag} → ${newTag}):`);

  // i18n translations.ts — hero.badge
  replaceInFile("site/src/i18n/translations.ts", new RegExp(escapeRegex(oldTag), "g"), newTag);
  console.log(`  ✓ site/src/i18n/translations.ts`);

  // Download.tsx — 下载链接与 release 标签
  replaceInFile("site/src/components/Download/Download.tsx", new RegExp(escapeRegex(oldTag), "g"), newTag);
  console.log(`  ✓ site/src/components/Download/Download.tsx`);

  // Hero.tsx — 平台版本徽章
  replaceInFile("site/src/components/Hero/Hero.tsx", new RegExp(escapeRegex(oldTag), "g"), newTag);
  console.log(`  ✓ site/src/components/Hero/Hero.tsx`);
}

console.log(`\n✅ 全部版本号已统一为 ${newVersion}\n`);
