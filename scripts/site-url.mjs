// Resolve the absolute base URL the deployed site is served from, and substitute
// it for `%SITE_URL%` placeholders in index.html.
//
// Why: social crawlers (Open Graph / Twitter) discard relative URLs, so og:image
// and og:url must be absolute in production. Locally we default to "" so every
// path stays relative — the built dist/ then works offline on localhost and,
// because the paths are relative to the page, also under the GitHub Pages
// `/<repo>/` subpath. CI sets SITE_URL to the real Pages URL for the deploy.
//
//   SITE_URL=https://user.github.io/repo/ node scripts/build-web.mjs

/** The base URL, normalized to end in a single "/" (or "" when unset). */
export function siteUrl() {
    let url = (process.env.SITE_URL ?? "").trim();
    if (url && !url.endsWith("/")) url += "/";
    return url;
}

/** Replace every `%SITE_URL%` in `html` with the resolved base URL. */
export function applySiteUrl(html) {
    return html.replaceAll("%SITE_URL%", siteUrl());
}
