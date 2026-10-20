import { describe, expect, it } from "vitest";
import { fuzzyScore } from "./fuzzy";

describe("fuzzyScore", () => {
  it("matches an in-order subsequence case-insensitively", () => {
    expect(fuzzyScore("WEB", "web01")).not.toBeNull();
    expect(fuzzyScore("wb1", "web01")).not.toBeNull();
    expect(fuzzyScore("bew", "web01")).toBeNull();
  });

  it("matches everything for an empty query", () => {
    expect(fuzzyScore("  ", "anything")).toBe(0);
  });

  it("ranks consecutive and word-start matches higher", () => {
    const exact = fuzzyScore("web", "web01")!;
    const inWord = fuzzyScore("web", "dev-webserver")!;
    const scattered = fuzzyScore("web", "w-e-b")!;
    expect(exact).toBeGreaterThan(inWord);
    expect(inWord).toBeGreaterThan(scattered);
  });

  it("ignores spaces in the query", () => {
    expect(fuzzyScore("new ssh", "New SSH session")).not.toBeNull();
  });
});
