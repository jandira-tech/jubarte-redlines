// Release facts the site prints, read from data/facts.jsonl: the GitHub
// release (assets), the engine CHANGELOG (dates), the npm package jubarte-wasm
// and the App Store listing id6790926615. scripts/sync-release.ts appends a new
// engine release's facts; anything else changes with scripts/facts.py.

import { fact } from "./facts.ts";

export const ENGINE_VERSION: string = fact("engine.version");
export const ENGINE_RELEASED: string = fact("engine.released");
export const ENGINE_REPO: string = fact("engine.repo");
export const RELEASE_URL: string = `${ENGINE_REPO}/releases/tag/v${ENGINE_VERSION}`;
export const RELEASE_DL: string = `${ENGINE_REPO}/releases/download/v${ENGINE_VERSION}`;
export const CHANGELOG_URL: string = `${ENGINE_REPO}/blob/main/CHANGELOG.md`;

/** The Mac App Store listing, as Apple serves it today. */
export type AppStoreListing = { url: string; version: string; price: string; minOs: string };
export const APP_STORE: AppStoreListing = {
  url: fact("app_store.url"),
  version: fact("app_store.version"),
  price: fact("app_store.price"),
  minOs: fact("app_store.min_os"),
};

/** The app version last submitted to Apple (app.release.*). While Apple
 * reviews it the store still sells APP_STORE.version; the day the store sells
 * it, `scripts/facts.py set app_store.version` is the one change that turns
 * every page's "in Apple's review" copy into the present tense. */
export type AppRelease = { version: string; submitted: string };
export const APP_RELEASE: AppRelease = {
  version: fact("app.release.version"),
  submitted: fact("app.release.submitted"),
};
export const appInReview = (release: AppRelease, store: AppStoreListing): boolean =>
  release.version !== store.version;
export const APP_IN_REVIEW: boolean = appInReview(APP_RELEASE, APP_STORE);
/** `review` while Apple reviews APP_RELEASE, `live` once the store sells it. */
export const byPhase = (review: string, live: string): string => (APP_IN_REVIEW ? review : live);

/** The app's next release, in this repository: a free download with five
 * free uses, each a redline or a PDF (src/paywall.js meters both), then the
 * Jubarte Pro Yearly subscription. */
export type NextApp = { freeUses: number; yearly: string };
export const APP_NEXT: NextApp = { freeUses: fact("app.free_uses"), yearly: fact("app.yearly") };

export const SUPPORT_EMAIL: string = fact("contact.support_email");
export const COUNSEL_EMAIL: string = fact("contact.counsel_email");
export const COMPANY: string = fact("company.name");

export type Archive = { target: string; file: string; size: number };

export const ARCHIVES: Archive[] = fact("release.archives");

export const WHEELS: Archive[] = fact("release.wheels");

/** An engine release: version, date (YYYY-MM-DD) and a one-line summary. */
export type Release = { v: string; d: string; t: string };
export const RELEASES: Release[] = fact("release.history");

/** Bytes as the site prints them. */
export function size(n: number): string {
  if (n < 1024) return `${n} B`;
  // Round first, so 1,048,575 bytes is "1.0 MB" rather than "1024 KB".
  const kb = Math.round(n / 1024);
  if (kb < 1024) return `${kb} KB`;
  return `${(n / 1048576).toFixed(1)} MB`;
}
