import "./test.css";
import { Suspense, createResource } from "solid-js";

export default function SubPage() {
	const [res] = createResource(() =>
		fetch("/api/message").then((resp) => resp.text()),
	);

	return (
		<div>
			<h1>Sub page</h1>
			<Suspense>
				<div>{res()}</div>
			</Suspense>
		</div>
	);
}
