import type { Group } from "./group";
import type { User } from "./user";

export type Identity = (Group & { type: "group" }) | (User & { type: "user" });
