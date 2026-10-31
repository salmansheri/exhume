import { greet } from "./greet";
import "./feature";
const { format } = require("./format");

export const main = () => greet() + format();
