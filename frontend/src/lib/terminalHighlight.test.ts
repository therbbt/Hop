import { describe, expect, it } from "vitest";
import { highlightTerminalOutput } from "./terminalHighlight";

const ESC = "\x1b";

describe("highlightTerminalOutput", () => {
  it("colors eza's day-first timestamp as one span", () => {
    expect(highlightTerminalOutput("18 Oct 20:24")).toBe(`${ESC}[0m${ESC}[90m18 Oct 20:24${ESC}[0m`);
  });

  it("colors ls's month-first timestamp as one span", () => {
    expect(highlightTerminalOutput("Oct 18 20:24")).toBe(`${ESC}[0m${ESC}[90mOct 18 20:24${ESC}[0m`);
    expect(highlightTerminalOutput("Jan  1  2024")).toBe(`${ESC}[0m${ESC}[90mJan  1  2024${ESC}[0m`);
  });

  it("keeps a syslog timestamp's seconds", () => {
    expect(highlightTerminalOutput("Oct 18 20:24:13 host")).toBe(`${ESC}[0m${ESC}[90mOct 18 20:24:13${ESC}[0m host`);
  });

  it("restores the program's own color after a highlighted span", () => {
    // `ip -c addr`-style: the program colors a whole field, and only part
    // of it matches a rule - the rest must keep the program's color.
    const out = highlightTerminalOutput(`${ESC}[34mhost 10.0.0.1 up${ESC}[0m`);
    expect(out).toBe(`${ESC}[34mhost ${ESC}[0m${ESC}[36m10.0.0.1${ESC}[0m${ESC}[34m up${ESC}[0m`);
  });

  it("doesn't restore a color the program already reset", () => {
    const out = highlightTerminalOutput(`${ESC}[34mx${ESC}[0m 10.0.0.1`);
    expect(out).toBe(`${ESC}[34mx${ESC}[0m ${ESC}[0m${ESC}[36m10.0.0.1${ESC}[0m`);
  });
});
