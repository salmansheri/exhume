"use client";

import { useCallback, useEffect, useRef, useState } from "react";
import type { Stratum } from "../fixtures";
import { Breakable } from "./breakable";

const RISE_DELAY = 500;
const RISE_STAGGER = 420;
const VERDICT_DELAY = 700;

type Props = {
  command: string;
  strata: Stratum[];
  summary: string;
  exit: number;
};

// The signature moment: findings rise out of the ground one by one, in report order, then the verdict lands.
export function DigSite({ command, strata, summary, exit }: Props) {
  const findings = strata.flatMap((s) => s.files).filter((f) => f.finding).map((f) => f.label);
  const findingCount = findings.length;
  const [risen, setRisen] = useState(0);
  const [done, setDone] = useState(false);
  const groundRef = useRef<HTMLElement>(null);
  const timers = useRef<ReturnType<typeof setTimeout>[]>([]);

  const scan = useCallback(() => {
    timers.current.forEach(clearTimeout);
    const reduce = window.matchMedia("(prefers-reduced-motion: reduce)").matches;
    if (reduce) {
      setRisen(findingCount);
      setDone(true);
      return;
    }
    setRisen(0);
    setDone(false);
    timers.current = Array.from({ length: findingCount }, (_, i) =>
      setTimeout(() => setRisen(i + 1), RISE_DELAY + i * RISE_STAGGER),
    );
    timers.current.push(setTimeout(() => setDone(true), VERDICT_DELAY + findingCount * RISE_STAGGER));
  }, [findingCount]);

  useEffect(() => {
    const ground = groundRef.current;
    if (!ground || !("IntersectionObserver" in window)) {
      scan();
      return;
    }
    const observer = new IntersectionObserver(
      ([entry]) => {
        if (entry.isIntersecting) {
          scan();
          observer.disconnect();
        }
      },
      { rootMargin: "0px 0px -25% 0px" },
    );
    observer.observe(ground);
    const scheduled = timers;
    return () => {
      observer.disconnect();
      scheduled.current.forEach(clearTimeout);
    };
  }, [scan]);

  return (
    <section className="ground" ref={groundRef} aria-labelledby="ground-command">
      <div className="ground-head">
        <code id="ground-command">
          $ <Breakable text={command} />
        </code>
        <button type="button" className="key key-sm" onClick={scan} aria-busy={!done}>
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" aria-hidden="true">
            <path d="M3 12a9 9 0 1 0 3-6.7L3 8" />
            <path d="M3 3v5h5" />
          </svg>
          Scan again
        </button>
      </div>
      {strata.map((stratum) => (
        <div className="stratum" key={stratum.path}>
          <div className="pkg">
            {stratum.name}
            <span>{stratum.path}</span>
          </div>
          <ul className="files">
            {stratum.files.map((file) => {
              if (!file.finding) {
                return (
                  <li className="file" key={file.label}>
                    <span>
                      <Breakable text={file.label} />
                    </span>
                  </li>
                );
              }
              const order = findings.indexOf(file.label) + 1;
              return (
                <li className="file finding" key={file.label} data-risen={order <= risen ? "" : undefined}>
                  <span>
                    <Breakable text={file.label} />
                  </span>
                  <span className="confidence">high</span>
                </li>
              );
            })}
          </ul>
        </div>
      ))}
      <div className="verdict" data-pending={done ? undefined : ""} aria-live="polite">
        <span>{summary}</span>
        <span className="keycap">
          exit <b>{exit}</b>
        </span>
      </div>
    </section>
  );
}
