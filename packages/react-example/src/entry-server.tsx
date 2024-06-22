import React from "react";
import {
	renderToString,
	renderToReadableStream,
} from "react-dom/server.browser";
import App from "./app";

const decoder = new TextDecoder();

export default async function render() {
	const html = await renderToReadableStream(
		<React.StrictMode>
			<App />
		</React.StrictMode>,
	);

	const reader = html.getReader();

	const output = [];

	while (true) {
		const { done, value } = await reader.read();
		if (value) {
			output.push(...value);
		}
		if (done) {
			break;
		}
	}

	const buffer = new Uint8Array(output);

	return { content: decoder.decode(buffer.buffer), head: [] };
}
