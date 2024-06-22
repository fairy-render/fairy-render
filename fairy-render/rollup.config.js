import ts from "rollup-plugin-ts";
import resolve from "@rollup/plugin-node-resolve";
import commonjs from "@rollup/plugin-commonjs";

export default [
	{
		input: {
			stream: "srcjs/stream.js",
		},
		output: {
			dir: "src/",
			format: "es",
		},
		external: ["@klaver/http", "@klaver/base", "util", "inherits", "events"],
		plugins: [
			resolve({
				preferBuiltins: false,
			}),
			commonjs({
				requireReturnsDefault: false, ///["util"],
				esmExternals: "namespace",
			}),
		],
	},
	{
		input: {
			util: "srcjs/util.js",
		},
		output: {
			dir: "src/",
			format: "es",
		},
		external: ["@klaver/http", "@klaver/base"],
		plugins: [
			resolve({
				preferBuiltins: false,
			}),
			commonjs(),
		],
	},
	{
		input: {
			inherits: "srcjs/inherits.js",
		},
		output: {
			dir: "src/",
			format: "es",
		},
		external: ["util"],
		plugins: [
			resolve({
				preferBuiltins: false,
			}),
			commonjs(),
		],
	},
	{
		input: {
			events: "srcjs/events.js",
		},
		output: {
			dir: "src/",
			format: "es",
		},
		external: ["util"],
		plugins: [
			resolve({
				preferBuiltins: false,
			}),
			commonjs(),
		],
	},
];
