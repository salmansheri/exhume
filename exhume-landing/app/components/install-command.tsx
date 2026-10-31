"use client";

import { useEffect, useRef, useState } from "react";
import { Breakable } from "./breakable";

const COMMAND = "cargo install --git https://github.com/salmansheri/exhume exhume-cli";

type CopyState = "idle" | "copied" | "manual";

export function InstallCommand({ note }: { note: React.ReactNode }) {
  const [state, setState] = useState<CopyState>("idle");
  const wellRef = useRef<HTMLElement>(null);
  const timer = useRef<ReturnType<typeof setTimeout>>(undefined);

  useEffect(() => () => clearTimeout(timer.current), []);

  function settle(next: CopyState) {
    setState(next);
    clearTimeout(timer.current);
    timer.current = setTimeout(() => setState("idle"), 2000);
  }

  // When the clipboard is unavailable, select the command so the visitor can copy it by hand.
  function selectCommand() {
    const well = wellRef.current;
    const selection = window.getSelection();
    if (!well || !selection) return;
    const range = document.createRange();
    range.selectNodeContents(well.querySelector("[data-command]") ?? well);
    selection.removeAllRanges();
    selection.addRange(range);
  }

  // Older or locked-down browsers lack the async clipboard API; fall back to a hidden textarea copy.
  function legacyCopy() {
    const area = document.createElement("textarea");
    area.value = COMMAND;
    area.setAttribute("readonly", "");
    area.style.position = "fixed";
    area.style.opacity = "0";
    document.body.appendChild(area);
    area.select();
    let ok = false;
    try {
      ok = document.execCommand("copy");
    } catch {
      ok = false;
    }
    area.remove();
    return ok;
  }

  async function copy() {
    try {
      await navigator.clipboard.writeText(COMMAND);
      settle("copied");
    } catch {
      if (legacyCopy()) {
        settle("copied");
      } else {
        selectCommand();
        settle("manual");
      }
    }
  }

  const label = state === "copied" ? "Copied" : state === "manual" ? "Selected, copy it" : "Copy";

  return (
    <div className="install">
      <p className="install-note">Install from source today (needs Rust)</p>
      <div className="install-row">
        <code className="well" ref={wellRef}>
          <span className="prompt">$ </span>
          <span data-command>
            <Breakable text={COMMAND} />
          </span>
        </code>
        <button type="button" className="key key-primary" data-state={state === "idle" ? undefined : "copied"} onClick={copy}>
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" aria-hidden="true">
            {state === "copied" ? (
              <path d="M20 6 9 17l-5-5" />
            ) : (
              <>
                <rect x="9" y="9" width="12" height="12" rx="2" />
                <path d="M5 15V5a2 2 0 0 1 2-2h10" />
              </>
            )}
          </svg>
          <span aria-live="polite">{label}</span>
        </button>
      </div>
      <p className="install-note">{note}</p>
    </div>
  );
}
