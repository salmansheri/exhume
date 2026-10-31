import type { Shape } from "./types";
import { type Color } from "./types";
import greet from "./greeting";
import { named } from "./other";
import { forwarded } from "./forward";

export type { Size } from "./types";
export const main = (shape: Shape, color: Color) => greet() + named + forwarded;
