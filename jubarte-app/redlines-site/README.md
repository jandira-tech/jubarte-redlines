# redlines-site: the old redline tool's addresses

The standalone web redliner that lived here is retired. jubarte.pro/demo runs
the same engine as WebAssembly in the visitor's tab, so this Worker only sends
the tool's old addresses there, with a permanent (301) redirect:

| Host                   | Zone          |
| ---------------------- | ------------- |
| `redlines.free`        | redlines.free |
| `www.redlines.free`    | redlines.free |
| `redlines.jubarte.pro` | jubarte.pro   |
| `redlines.arthur.law`  | arthur.law    |

| Path on those hosts           | Goes to                        |
| ----------------------------- | ------------------------------ |
| `/privacy`, `/privacy.html`   | `https://jubarte.pro/privacy`  |
| `/terms`, `/terms.html`       | `https://jubarte.pro/terms`    |
| anything else                 | `https://jubarte.pro/demo`     |

Each host is a custom domain in `wrangler.jsonc`; `pnpm run deploy` attaches
them. jubarte.pro's Worker (`../jubarte-site`) must never list one of them: two
Workers claiming a domain take it from each other on every deploy, and
`jubarte-site/test/node/domains.test.ts` fails if they do.

## Commands

```sh
pnpm install
pnpm test         # the redirects, in workerd
pnpm run lint
pnpm run typecheck
pnpm run deploy   # wrangler deploy
```

## What is gone

The tool's pages, its wasm bundle, its quota API and the binding to its D1
database `jubarte-redlines-quota` were removed on 2 October 2026; git history
keeps them. The database itself is still in the Cloudflare account, holding
anonymous visitor counters. Delete it from the dashboard when nobody needs
them.
