import { used } from "./direct";
import { fromChain, renamed as alsoRenamed } from "./barrel";

export const main = () => used + fromChain + alsoRenamed;
