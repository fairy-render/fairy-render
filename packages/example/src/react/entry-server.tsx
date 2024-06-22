
import React from "react";
import { renderToString } from "react-dom/server.browser";
import App from "./app";


export default function render() {
	const html = renderToString(
		<React.StrictMode>
			<App />
		</React.StrictMode>,
	);
	return { content:html, head: [] };
}

