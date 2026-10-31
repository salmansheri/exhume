import debounce from "lodash/debounce";
import { thing } from "@scope/pkg/sub";
import { readFileSync } from "node:fs";
import path from "path";

export const main = () => [debounce, thing, readFileSync, path];
