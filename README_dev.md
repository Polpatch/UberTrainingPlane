Allenamento WASM — Sviluppo locale

Prerequisiti:
- Rust (stable)
- cargo
- trunk (`cargo install trunk`)

Build e sviluppo:

```bash
# build release (output in app/dist)
cd app
trunk build --release

# serve in dev mode
trunk serve
```

Deployment:
- Il workflow GitHub Actions `/.github/workflows/gh-pages.yml` costruisce l'app e pubblica `app/dist` su GitHub Pages del repository.

Note:
- Questo repository usa Yew (WASM).
- Il frontend ora supporta upload di una scheda JSON, visualizza la struttura del giorno e l'esercizio corrente, e può salvare serie in `localStorage`.
- Il timer di recupero è implementato come prototipo. Prossimi passi: migliorare il flusso di sessione e il deploy GitHub Pages.
