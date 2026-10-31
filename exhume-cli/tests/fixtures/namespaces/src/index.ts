import * as math from "./math";
import type * as types from "./types";
import * as passed from "./passed";
import * as computed from "./computed";
import * as spread from "./spread";
import * as picked from "./picked";

const register = (mod: object) => Object.keys(mod).length;
const key = "x";

export const main = (shape: types.Shape) =>
  math.add(1, 2) + register(passed) + computed[key] + { ...spread }.z;

const { one } = picked;
export const first = one;
