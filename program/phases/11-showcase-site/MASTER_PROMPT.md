# Phase 11: showcase site on Vercel (MASTER PROMPT, optional)

You lead Phase 11, running in your own chat and worktree: a public showcase page for the release.

## Goal
A static site that has:
- a hero video or animation
- a gallery of every cluster (renders and short loops)
- the **WASM/WebGPU configurator demo**, playable in the browser with synthetic telemetry (Space and 1–9)
- a download button pointing at the GitHub Release

It is built from the same Rust/WASM code, with no framework needed. The page logic is Rust compiled to WASM; HTML and CSS are the only non-Rust parts (ONBOARDING rule 8).

## Steps
1. **Design** in Penpot on page `11 · Showcase` (`docs/design/PENPOT.md`), reviewed against a fixed yes/no checklist, with TJX's direction check before the build. No scores and no critic loops (AGENTS.md rule 12).
2. **Build** a `site/` output from `tools/web-preview` and the configurator crates (`cargo run -p site-build`, a new tool that this phase adds; it does not exist yet), compressing renders to WebP or AVIF and videos to MP4/WebM.
3. **Deploy with Vercel:**
   - The `vercel` plugin is not enabled (checked 2026-11-27): ask TJX to enable it before this step.
   - Use the `vercel:vercel-cli` and `vercel:deploy` skills. Check login with `vercel whoami`.
   - **Not logged in:** stop and ask TJX to log in (`vercel login`). Never handle credentials yourself.
   - Deploy a preview, verify it with `vercel:verification` and the built-in browser, then deploy to production.
   - Set caching headers for `.wasm` and assets (`vercel:cdn-caching`).
   - The Vercel Workflow DevKit (`vercel:workflow`) is for durable backend workflows and has no use on a static site, so don't use it.
4. Add the site URL to the README and the GitHub Release notes.

## Exit criteria
- The production URL works.
- The WebGPU demo runs, with a fallback message for browsers without WebGPU.
- Lighthouse performance is 90 or above on desktop.
- The README is updated.
