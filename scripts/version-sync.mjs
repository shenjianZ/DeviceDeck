/**
 * 版本同步脚本
 * 将 package.json 的版本号同步到 Cargo.toml 和 tauri.conf.json
 *
 * 用法：
 *   pnpm version 0.2.0        ← 自动触发（通过 version lifecycle hook）
 *   node scripts/version-sync.mjs 0.2.0  ← 手动执行
 */
import { readFileSync, writeFileSync } from "node:fs";
import { resolve, dirname } from "node:path";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");

const files = {
  cargo: resolve(root, "src-tauri", "Cargo.toml"),
  tauri: resolve(root, "src-tauri", "tauri.conf.json"),
};

function main() {
  const newVersion = process.argv[2];
  if (!newVersion) {
    console.error("用法: node scripts/version-sync.mjs <version>");
    process.exit(1);
  }

  if (!/^\d+\.\d+\.\d+(-[\w.]+)?$/.test(newVersion)) {
    console.error(`无效版本号: ${newVersion}，期望格式: x.y.z 或 x.y.z-tag`);
    process.exit(1);
  }

  // 同步 Cargo.toml — 匹配 version = "... 并整行替换
  let cargoContent = readFileSync(files.cargo, "utf-8");
  const cargoOld = cargoContent.match(/version\s*=\s*"[^"]*/)?.[0]?.replace('version = "', "") ?? "unknown";
  cargoContent = cargoContent.replace(
    /^(version\s*=\s*")[^"]*(.*)/m,
    `$1${newVersion}$2`,
  );
  writeFileSync(files.cargo, cargoContent, "utf-8");
  console.log(`Cargo.toml: ${cargoOld} → ${newVersion}`);

  // 同步 tauri.conf.json
  const tauriContent = readFileSync(files.tauri, "utf-8");
  const tauriJson = JSON.parse(tauriContent);
  const oldTauri = tauriJson.version;
  tauriJson.version = newVersion;
  writeFileSync(files.tauri, JSON.stringify(tauriJson, null, 2) + "\n", "utf-8");
  console.log(`tauri.conf.json: ${oldTauri} → ${newVersion}`);

  console.log(`\n✓ 版本已同步到 ${newVersion}`);
}

main();
