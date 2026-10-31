const page = await import("./page");
const { render } = await import(`./render`);
require("./polyfill.cjs");

export const main = () => render(page.title);
export const theme = () => import("./theme").then((m) => m.dark);
