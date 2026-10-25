import { Button } from "@acme/ui";
import { internal } from "@acme/ui/src/internal";
import { format } from "@acme/utils";
import { extra } from "@acme/utils/src/extra";
import { greet } from "@/lib/aliased";

export const app = () => Button(format(greet() + extra + internal));
