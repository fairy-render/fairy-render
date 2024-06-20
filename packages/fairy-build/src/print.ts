import { resolveRuntimeConfig } from "./config.js";
import { createRuntimeConfigJson, loadConfig, type Options } from "./shared.js";

export default async function print(options: Options) {
	const cfg = await loadConfig(options.config);
	const config = resolveRuntimeConfig(await cfg.resolve(), options);

	createRuntimeConfigJson(config);
}
