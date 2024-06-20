import { writeFile } from "node:fs/promises";
import { resolveRuntimeConfig } from "./config.js";
import { createRuntimeConfigJson, loadConfig, type Options } from "./shared.js";

export default async function print(options: Options & { output?: string }) {
	const cfg = await loadConfig(options.config);
	const config = resolveRuntimeConfig(await cfg.resolve(), options);

	const output = createRuntimeConfigJson(config);

	const json = JSON.stringify(output, null, 2);

	if (options.output) {
		try {
			await writeFile(options.output, json);
		} catch (e) {
			console.error("could not write file: ", e.message);
		}
	} else {
		console.log(json);
	}
}
