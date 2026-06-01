# Uber Training Plane — Stato attuale

## Descrizione
Questa repository contiene un’applicazione single-page in Rust + Yew per il tracciamento degli allenamenti, compilata in WebAssembly e servita con Trunk.

La versione attuale è contenuta nella cartella `app/` e offre:
- caricamento di una scheda di allenamento da file JSON
- visualizzazione dei giorni e degli esercizi
- input per peso e ripetizioni
- registrazione manuale delle serie
- timer di recupero con countdown
- salvataggio automatico della serie al termine del timer
- persistenza locale dei dati in `LocalStorage`

## Struttura del progetto
- `app/` — crate Rust/Yew principale con l’applicazione WASM
  - `app/src/lib.rs` — logica dell’app e componenti Yew
  - `app/index.html` — entrypoint HTML per Trunk
  - `app/Cargo.toml` — manifest crate
- `schede/` — esempi di schede JSON di allenamento
- `static/` — eventuali asset statici di progetto

## Stato attuale
- L’app compila correttamente con `trunk build`
- Il timer di recupero parte e decrementa regolarmente
- Al termine del timer la serie viene salvata in `LocalStorage`
- Il caricamento dei file JSON funziona correttamente
- La selezione del giorno carica la sessione salvata associata a quel giorno

## Come eseguire
```bash
cd app
trunk serve
```

Poi aprire l’app nel browser all’URL mostrato da Trunk.

Per una build di produzione:
```bash
cd app
trunk build --release
```

## Note
- Ho rimosso i file root `README.md` e `index.html` precedenti perché non fanno più parte del flusso dell’app.
- Il punto di ingresso dell’app resta `app/index.html` e non la root del repository.
