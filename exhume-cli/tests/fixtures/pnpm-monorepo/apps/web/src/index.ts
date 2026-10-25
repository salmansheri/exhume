import { Button } from "@acme/ui";
import { format } from "@acme/utils";
import { greet } from "@/lib/aliased";

export const app = () => Button(format(greet()));
