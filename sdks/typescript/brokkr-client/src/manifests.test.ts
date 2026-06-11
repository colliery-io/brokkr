import { mkdtempSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { describe, expect, it } from "vitest";

import { readManifests, sha256Hex } from "./index.js";

function tmp(): string {
  return mkdtempSync(join(tmpdir(), "brokkr-manifests-"));
}

describe("readManifests (BROKKR-T-0197)", () => {
  it("concatenates a folder in sorted-name order, skipping non-yaml", async () => {
    const dir = tmp();
    writeFileSync(join(dir, "02-cm.yaml"), "apiVersion: v1\nkind: ConfigMap\nmetadata:\n  name: c\n");
    writeFileSync(join(dir, "01-ns.yaml"), "apiVersion: v1\nkind: Namespace\nmetadata:\n  name: n\n");
    writeFileSync(join(dir, "notes.txt"), "ignored");
    const stream = await readManifests(dir);
    expect(stream.indexOf("kind: Namespace")).toBeLessThan(stream.indexOf("kind: ConfigMap"));
    expect(stream).toContain("\n---\n");
    expect(stream).not.toContain("ignored");
  });

  it("accepts a single multi-doc file", async () => {
    const dir = tmp();
    const f = join(dir, "all.yaml");
    writeFileSync(f, "apiVersion: v1\nkind: Namespace\nmetadata:\n  name: a\n---\napiVersion: v1\nkind: ConfigMap\nmetadata:\n  name: b\n");
    const stream = await readManifests(f);
    expect(stream).toContain("kind: Namespace");
    expect(stream).toContain("kind: ConfigMap");
  });

  it("rejects a document missing apiVersion or kind", async () => {
    const dir = tmp();
    writeFileSync(join(dir, "bad.yaml"), "kind: ConfigMap\nmetadata:\n  name: x\n");
    await expect(readManifests(dir)).rejects.toThrow(/apiVersion and kind/);
  });

  it("errors on an empty directory and a missing path", async () => {
    const dir = tmp();
    await expect(readManifests(dir)).rejects.toThrow(/no .yaml/);
    await expect(readManifests(join(dir, "nope"))).rejects.toThrow(/path not found/);
  });
});

describe("sha256Hex", () => {
  it("matches the known empty-string vector and is stable", async () => {
    expect(await sha256Hex("")).toBe(
      "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
    );
    const a = "apiVersion: v1\nkind: ConfigMap\n";
    expect(await sha256Hex(a)).toBe(await sha256Hex(a));
    expect(await sha256Hex(a)).not.toBe(await sha256Hex("apiVersion: v1\nkind: Secret\n"));
  });
});
