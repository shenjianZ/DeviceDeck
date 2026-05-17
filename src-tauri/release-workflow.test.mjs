import { readFileSync } from "node:fs";
import { join } from "node:path";
import { describe, expect, test } from "vitest";

function readReleaseWorkflow() {
  return readFileSync(join(process.cwd(), ".github", "workflows", "release.yml"), "utf8");
}

function buildJobSection(workflow) {
  const start = workflow.indexOf("  build:");
  const end = workflow.indexOf("  merge-updater-manifest:");

  expect(start).toBeGreaterThanOrEqual(0);
  expect(end).toBeGreaterThan(start);

  return workflow.slice(start, end);
}

describe("Release workflow bundled binaries", () => {
  test("fetches Git LFS objects before building release assets", () => {
    const buildJob = buildJobSection(readReleaseWorkflow());

    expect(buildJob).toMatch(/uses:\s*actions\/checkout@v4[\s\S]*?lfs:\s*true/);
    expect(buildJob).toContain('git lfs pull --include="src-tauri/binaries/**"');
  });

  test("fails release builds when bundled binaries are still Git LFS pointers", () => {
    const buildJob = buildJobSection(readReleaseWorkflow());

    expect(buildJob).toContain("Validate bundled binary contents");
    expect(buildJob).toContain("git-lfs.github.com/spec/v1");
    expect(buildJob).toContain("src-tauri/binaries");
  });
});
