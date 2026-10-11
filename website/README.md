# Slipstream website

Production: https://slipstream-dj.vercel.app/

A static landing page and browser sound study. The demo synthesizes original loops with Web Audio; it does not run the native Rust/Qt mixing engine. Audio starts only after pressing Play.

## Local preview

From this directory:

```sh
npm run dev
```

Open http://127.0.0.1:4173. No dependencies or build step are required. Run `npm run check` to check JavaScript syntax.

## Deploy

The current public deployment is hosted in Rayhan’s `rayhanadev-team` Vercel scope, under the `slipstream-dj` project. It is deployed manually. Maintainers with access to that Vercel project can run the following from this directory:

```sh
vercel link --yes --project slipstream-dj --scope rayhanadev-team
vercel deploy --prod --yes --scope rayhanadev-team
```

Select or retain the Other framework preset, `.` output directory, and no build command. Git auto-deployment is disconnected so changes to the native application do not replace this static site. If the CLI reconnects the repository while linking, run `vercel git disconnect --scope rayhanadev-team` before relying on manual deployment.

For your own deployment, use a Vercel project and scope you control instead. Update the canonical URL and Open Graph URLs in `index.html` to your deployment URL or custom domain.

## Files

- `index.html`: page content, accessible controls, metadata.
- `style.css`: responsive layout and visual design.
- `app.js`: controls, animated decorative waveforms, playback status.
- `audio.js`: original synthesized loops, tempo, crossfader, filter.
- `assets/`: icon, share image, and self-hosted OFL fonts with license notices.

The project CTA links to the public repository. There is no checkout, email collection, analytics, or backend.
