import { used } from "./util";

export const load = (path: string) => [import(path), used];
