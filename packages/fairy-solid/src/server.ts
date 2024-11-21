import {
  Component,
  ComponentProps,
  sharedConfig,
  lazy as solidLazy,
} from "solid-js";
import type { RequestEvent } from "solid-js/web";
import {
  isServer,
  renderToStringAsync,
  RequestContext,
  generateHydrationScript,
} from "solid-js/web";

class AsyncLocalStorage<T> {
  #init?: T;
  run<U>(init: T, cb: () => U) {
    this.#init = init;
    return cb();
  }

  getStore() {
    return this.#init;
  }
}

// using global on a symbol for locating it later and detaching for environments that don't support it.
function provideRequestEvent<T extends RequestEvent, U>(
  init: T,
  cb: () => U
): U {
  if (!isServer)
    throw new Error("Attempting to use server context in non-server build");
  // biome-ignore lint/suspicious/noAssignInExpressions: <explanation>
  const ctx: AsyncLocalStorage<T> = ((globalThis as any)[RequestContext] =
    (globalThis as any)[RequestContext] || new AsyncLocalStorage<T>());
  return ctx.run(init, cb);
}

export async function render<A>(req: Request, component: () => A) {
  const event = {
    request: req,
    response: new Response(),
  };

  return provideRequestEvent(event, async () => {
    // @ts-ignore
    sharedConfig.context = { event };
    const resp = await renderToStringAsync(component);
    return {
      content: resp,
      head: [generateHydrationScript()],
    };
  });
}
