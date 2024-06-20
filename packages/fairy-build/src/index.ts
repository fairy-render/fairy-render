import { FairyConfig, type UserConfig } from "./config.js";

import build from "./build.js";
import watch from "./watch.js";

export type { UserConfig } from "./config.js";

export { watch, build };

export function defineConfig(cfg: () => UserConfig): FairyConfig {
	return new FairyConfig(cfg);
}
