// The old free redline tool's hosts. The tool itself is gone: jubarte.pro/demo
// runs the same engine in the visitor's tab, so every address it had sends
// people there for good. Its policies have their jubarte.pro counterparts.

const SITE = "https://jubarte.pro";

/** Where a path on an old host now lives. */
export function destination(url: URL): string {
  const page = url.pathname.replace(/\.html$/, "").replace(/\/+$/, "");
  if (page === "/privacy" || page === "/terms") return `${SITE}${page}`;
  return `${SITE}/demo`;
}

export default {
  async fetch(request: Request): Promise<Response> {
    return new Response(null, {
      status: 301,
      headers: { location: destination(new URL(request.url)) },
    });
  },
} satisfies ExportedHandler;
