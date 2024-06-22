import { render } from "@fairy-render/solid/server";
import App from "./app.jsx";

export default async function server(req: Request) {
	return await render(req, () => <App url={req.url} />);
}
