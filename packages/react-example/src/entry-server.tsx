// import React from "react";
// import {
// 	renderToString,
// 	renderToReadableStream,
// } from "react-dom/server.browser";
import render from "@fairy-render/react/server";
import { App } from "./app";
import React from "react";

const decoder = new TextDecoder();

export default async function main() {
	const html = await render(
		<React.StrictMode>
			<App />
		</React.StrictMode>,
		{},
	);

	return html;
	// const reader = html.getReader();
	// const output = [];
	// while (true) {
	// 	const { done, value } = await reader.read();
	// 	if (value) {
	// 		output.push(...value);
	// 	}
	// 	if (done) {
	// 		break;
	// 	}
	// }
	// const buffer = new Uint8Array(output);
	// return { content: decoder.decode(buffer.buffer), head: [] };
}
