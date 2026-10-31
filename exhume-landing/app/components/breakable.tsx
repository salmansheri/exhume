import { Fragment } from "react";

// Mono paths and commands may wrap only at a space or after a single "/", never mid-token:
// `file.ts:1:1` stays whole, `pnpm-monorepo` never splits at its hyphen, and `https://` stays
// attached to the host that follows it.
export function Breakable({ text }: { text: string }) {
  return (
    <>
      {text.split(" ").map((word, w) => (
        <Fragment key={w}>
          {w > 0 && " "}
          {word.split(/(?<=[^/]\/)(?!\/)/).map((segment, i) => (
            <Fragment key={i}>
              {i > 0 && <wbr />}
              <span className="nowrap">{segment}</span>
            </Fragment>
          ))}
        </Fragment>
      ))}
    </>
  );
}
