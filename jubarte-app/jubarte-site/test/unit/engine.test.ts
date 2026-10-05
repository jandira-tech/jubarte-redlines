import { afterAll, beforeAll, describe, expect, it, vi } from "vitest";

// engine.js runs in the page: it reads <html data-engine>, builds the worker
// URL from location and starts a module Worker. A fake Worker lets the tests
// play the worker's side.
type Posted = { id: number; op: string; args: unknown };
type WorkerEvent = {
  data?: { id: number; ok: boolean; result?: unknown; error?: string };
  message?: string;
};

class FakeWorker {
  static all: FakeWorker[] = [];
  listeners = new Map<string, ((e: WorkerEvent) => void)[]>();
  posted: Posted[] = [];
  terminated = false;
  constructor(
    public url: URL,
    public opts: WorkerOptions,
  ) {
    FakeWorker.all.push(this);
  }
  addEventListener(type: string, fn: (e: WorkerEvent) => void) {
    this.listeners.set(type, [...(this.listeners.get(type) ?? []), fn]);
  }
  postMessage(msg: Posted) {
    this.posted.push(msg);
  }
  terminate() {
    this.terminated = true;
  }
  emit(type: string, e: WorkerEvent) {
    for (const fn of this.listeners.get(type) ?? []) fn(e);
  }
}

let mod: typeof import("../../site/static/js/engine.js");

beforeAll(async () => {
  vi.stubGlobal("document", { documentElement: { dataset: {} } });
  vi.stubGlobal("location", { origin: "https://jubarte.pro" });
  vi.stubGlobal("Worker", FakeWorker);
  mod = await import("../../site/static/js/engine.js");
});
afterAll(() => vi.unstubAllGlobals());

describe("engine", () => {
  it("starts one worker per build and settles calls from its replies", async () => {
    const e = mod.engine("slim");
    expect(mod.engine("slim")).toBe(e);
    const w = FakeWorker.all.at(-1) as FakeWorker;
    expect(w.url.searchParams.get("build")).toBe("slim");
    w.emit("message", { data: { id: 0, ok: true, result: "ready" } });
    const call = e.call("compare", {});
    const { id } = w.posted.at(-1) as Posted;
    w.emit("message", { data: { id, ok: true, result: 42 } });
    await expect(call).resolves.toBe(42);
  });

  it("retires a worker that failed to load, so the next call starts a new one", async () => {
    const first = mod.engine("full");
    const w = FakeWorker.all.at(-1) as FakeWorker;
    const pending = first.call("pdf", {});
    w.emit("error", { message: "engine-worker.js: 503" });
    await expect(pending).rejects.toThrow("503");
    expect(w.terminated).toBe(true);
    const second = mod.engine("full");
    expect(second).not.toBe(first);
    expect(FakeWorker.all.at(-1)).not.toBe(w);
  });

  it("retires a worker whose engine failed to start, instead of failing every call after", async () => {
    const first = mod.engine("full");
    const w = FakeWorker.all.at(-1) as FakeWorker;
    // A call queued before the failure gets no reply once the worker is gone.
    const queued = first.call("pdf", {});
    w.emit("message", { data: { id: 0, ok: false, error: "wasm fetch failed" } });
    await expect(queued).rejects.toThrow("wasm fetch failed");
    expect(w.terminated).toBe(true);
    expect(mod.engine("full")).not.toBe(first);
  });

  it("lets a retired engine's late error leave its replacement alone", () => {
    const old = mod.engine("slim");
    const w = FakeWorker.all.find((x) => x.url.searchParams.get("build") === "slim") as FakeWorker;
    w.emit("error", { message: "boom" });
    const fresh = mod.engine("slim");
    w.emit("error", { message: "again" });
    expect(mod.engine("slim")).toBe(fresh);
    expect(fresh).not.toBe(old);
  });
});
