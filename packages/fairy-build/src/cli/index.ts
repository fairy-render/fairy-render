import { program } from "commander";
import build from "../build.js";
import watch from "../watch.js";
import print from "../print.js";

export default function main() {
	program
		.command("build")
		.option("-c, --config <path>", "config path", "")
		.action(async (opts) => {
			await build(opts);
		});

	program
		.command("watch")
		.option("-c, --config <path>", "config path", "")
		.option("-p, --port <port>", "port", "3768")
		.action(async (opts) => {
			await watch(opts);
		});

	program
		.command("print")
		.option("-c, --config <path>", "config path", "")
		.option("-p, --port <port>", "port", "3768")
		.action(async (opts) => {
			await print(opts);
		});

	program.parse();
}
