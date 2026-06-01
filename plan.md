# Piano locale: Reinventare app tracciamento allenamenti

Obiettivo: costruire una webapp mobile-first in Rust/WASM (Yew) che carichi schede da file JSON, fornisca UI focalizzata sull'esercizio corrente (inserimento rapido peso/reps, timer di recupero) e salvi sessioni in `LocalStorage`.

## Stato attuale
- Progetto principale in `app/` già funzionante.
- Upload file JSON e parsing dei workout implementati.
- Visualizzazione dei giorni e selezione esercizi attiva.
- Input peso/reps funzionanti.
- Timer di recupero con countdown funzionante.
- Registrazione manuale delle serie e salvataggio in `LocalStorage` funzionanti.
- Al termine del timer la serie viene salvata automaticamente.
- Caricamento della sessione locale al cambio giorno funziona.
- `trunk build` esegue correttamente.

## Fatto finora
1. Scaffold Rust/Yew in `app/`.
2. Definizione del modello dati `Workout/Day/Exercise/CompletedSet`.
3. Implementazione parser JSON e interfaccia upload.
4. Implementazione timer di recupero e salvataggio locale.
5. Pulizia del repository eliminando file root obsoleti.

## Prossime modifiche
1. Caricamento schede anche da un DB NoSQL esterno.
2. Aggiungere un catalog dinamico delle schede disponibili.
3. Migliorare UI mobile e gestione delle sessioni.
4. Aggiungere deploy/CI/CD solo dopo che il flusso dati e la UI sono stabili.

## NoSQL e GitHub Pages
- GitHub Pages ospita solo contenuti statici, quindi non può eseguire un database o un backend server-side.
- Possiamo comunque usare un DB NoSQL esterno se esposto tramite un'API HTTP (Firebase, Supabase, MongoDB Atlas, ecc.).
- La app Yew può fare richieste `fetch` verso quell'API e caricare le schede dinamicamente.
- Se vuoi evitare problemi di sicurezza, è meglio usare un layer API o un servizio gestito che supporti CORS e regole di accesso.
- In pratica:
  - `GitHub Pages` serve i file statici della app.
  - la app nel browser fa `fetch` su un'API esterna per ottenere le schede.

## Rischi / considerazioni
- Accesso diretto a un DB da browser è possibile solo con servizi pensati per essere esposti pubblicamente.
- Serve autenticazione o regole di accesso se i dati non devono essere completamente pubblici.
- Se preferisci mantenere tutto statico, possiamo iniziare con un catalog JSON esterno e poi evolvere a un servizio API.

Created: 2026-06-01

