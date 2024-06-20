import { createServer, type InlineConfig } from "vite";
import { type Options, createConfig, loadConfig } from "./shared.js";
import { resolveRuntimeConfig } from "./config.js";

export default async function watch(options: Options) {
	const cfg = await loadConfig(options.config);

	const config = await createConfig(
		resolveRuntimeConfig(await cfg.resolve(), options),
		"client",
		"watch",
	);

	const server = await createServer(config);

	await server.listen();

	server.printUrls();
	server.bindCLIShortcuts({ print: true });
}
