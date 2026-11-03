import { describe, expect, it } from "vitest";
import { collapseHome, cwdFromTitle, expandHome, parseOsc7, pathWithin } from "./cwd";

describe("parseOsc7", () => {
  it("reads the path out of a file:// report", () => {
    expect(parseOsc7("file://arch/home/me/Work/hop")).toBe("/home/me/Work/hop");
    expect(parseOsc7("file:///srv/app")).toBe("/srv/app");
  });
  it("decodes percent-encoding", () => {
    expect(parseOsc7("file://h/home/me/my%20app")).toBe("/home/me/my app");
  });
  it("ignores anything else", () => {
    expect(parseOsc7("http://example.com/x")).toBeNull();
  });
});

describe("cwdFromTitle", () => {
  it("handles Arch and Debian style titles", () => {
    expect(cwdFromTitle("me@arch:~/Work/hop")).toBe("~/Work/hop");
    expect(cwdFromTitle("ubuntu@web-1: /srv/app")).toBe("/srv/app");
  });
  it("ignores titles that aren't a prompt", () => {
    expect(cwdFromTitle("vim notes.txt")).toBeNull();
    expect(cwdFromTitle("me@arch: running build")).toBeNull();
  });
});

describe("expandHome / collapseHome", () => {
  it("round-trips paths under home", () => {
    expect(expandHome("~/app", "/home/me")).toBe("/home/me/app");
    expect(expandHome("~", "/home/me/")).toBe("/home/me");
    expect(collapseHome("/home/me/app", "/home/me")).toBe("~/app");
    expect(collapseHome("/home/me", "/home/me")).toBe("~");
  });
  it("leaves other paths alone", () => {
    expect(expandHome("/srv/app", "/home/me")).toBe("/srv/app");
    expect(expandHome("~/app", null)).toBe("~/app");
    expect(collapseHome("/home/meow", "/home/me")).toBe("/home/meow");
  });
});

describe("pathWithin", () => {
  it("matches the directory itself and anything below it", () => {
    expect(pathWithin("/home/me/app", "/home/me/app")).toBe(true);
    expect(pathWithin("/home/me/app/web", "/home/me/app/")).toBe(true);
  });
  it("doesn't match a sibling sharing a prefix", () => {
    expect(pathWithin("/home/me/app2", "/home/me/app")).toBe(false);
    expect(pathWithin("/home/me", "/home/me/app")).toBe(false);
  });
});
