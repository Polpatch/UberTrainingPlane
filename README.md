# 🏋️ Piattaforma Allenamento Estremo

Web app statica per **GitHub Pages** — zero backend, zero dipendenze esterne.
Le schede di allenamento sono file JSON nel repository; i progressi, la cronologia e i dati inseriti vengono salvati interamente nel `localStorage` del browser.

---

## 📱 Come funziona l'app

### 1 · Selezione scheda
Nella sidebar sinistra vengono elencate tutte le schede presenti in `schede/catalog.json` (o, in assenza del file, un catalogo incorporato nel codice). È possibile filtrare per nome, mese e anno.

### 2 · Visualizzazione giorno
Selezionando una scheda appaiono i tab dei giorni (Push, Pull, Legs, ecc.).
Ogni esercizio mostra:

| Elemento | Descrizione |
|---|---|
| **Badge serie** | Numero di serie previste dalla scheda JSON |
| **Badge reps** | Ripetizioni previste dalla scheda JSON |
| **Badge recupero** | Tempo di riposo in minuti:secondi letto dal campo `recupero` del JSON |
| **Pallini serie** | Uno per ogni serie — clic sul prossimo per segnarlo ✓, clic sull'ultimo segnato per annullarlo |
| **Campo Peso** | Peso usato (kg), salvato in localStorage |
| **Campo Reps effettive** | Es. `8/8/7/6`, salvato in localStorage |
| **Campo Note** | Note libere sulla sessione, salvate in localStorage |
| **Note scheda** | Indicazioni tecniche dall'autore della scheda (campo `note` nel JSON, read-only) |

### 3 · Timer di riposo
Il pulsante **⏱ Riposo** apre un popup con:
- Anello SVG animato che conta alla rovescia
- Tempo base preso dal campo `recupero` dell'esercizio nel JSON (default 90 s)
- Bottoni **−15 s / +15 s** per regolazione al volo
- Preset rapidi: 1 min, 1:30, 2 min, 3 min
- Avvio automatico all'apertura
- Colore giallo sotto i 30 s, rosso sotto i 10 s, con **beep audio** alla scadenza

### 4 · Cronologia esercizio (inline)
In fondo a ogni scheda esercizio è presente un **drawer collassabile** che mostra la tabella storica dell'esercizio (data, peso, reps effettive, note) filtrata per scheda e giorno.
Si apre anche cliccando l'icona 🕐 in alto a destra sull'esercizio.

### 5 · Salvataggio sessione
Il pulsante **💾 Salva sessione** nel toolbar registra uno snapshot completo del giorno corrente (pesi, reps, note di tutti gli esercizi) nella cronologia globale, con data e ora.

### 6 · Cronologia sessioni
In fondo alla pagina compare automaticamente la sezione **📋 Cronologia sessioni** con le ultime 30 sessioni salvate, ordinate dalla più recente, con scheda, giorno e riepilogo pesi/reps.

### 7 · Reset e export
- **Reset** → cancella i dati correnti del giorno selezionato (chiede conferma)
- **Esporta** → scarica un file `allenamento-export.json` con tutti i progressi e la cronologia

---

## 📂 Struttura repository

```
/
├── index.html                     ← App principale (GitHub Pages entry point)
├── README.md
└── schede/
    ├── catalog.json               ← Indice di tutte le schede
    ├── ATHLON_01_05-2026.json
    ├── ATHLON_Hypertrophy_02_05-2026.json
    └── ...                        ← altre schede
```

---

## 🗂 Convenzione nomi file schede

```
schede/NOME_NUMERO_mese-anno.json
```

**Esempi validi:**
```
schede/ATHLON_01_05-2026.json
schede/ForzeBase_03_01-2027.json
schede/PushPullLegs_02_09-2026.json
```

> Il nome deve corrispondere esattamente a quello dichiarato in `catalog.json`.

---

## 📋 Struttura `catalog.json`

Il catalogo è l'indice che l'app legge all'avvio per popolare la sidebar.

```json
[
  {
    "file":    "schede/ATHLON_01_05-2026.json",
    "nome":    "ATHLON",
    "numero":  "01",
    "mese":    "05",
    "anno":    "2026",
    "versione": "2.9"
  },
  {
    "file":    "schede/ATHLON_Hypertrophy_02_05-2026.json",
    "nome":    "ATHLON_Hypertrophy",
    "numero":  "02",
    "mese":    "05",
    "anno":    "2026",
    "versione": "1.0"
  }
]
```

---

## 🏗 Struttura JSON di una scheda

Ogni scheda è un file JSON con la seguente struttura. **Tutti i campi contrassegnati con \* sono obbligatori.**

```json
{
  "scheda": {
    "nome":     "ATHLON",          // * nome visualizzato nell'app
    "numero":   "01",              // * numero progressivo (stringa a 2 cifre)
    "mese":     "05",              // * mese (01–12, stringa a 2 cifre)
    "anno":     "2026",            // * anno (YYYY)
    "autore":   "Coach Estremo",   //   opzionale
    "versione": "2.9",             //   opzionale, mostrata nell'header

    "giorni": [
      {
        "id":   "g1",              // * identificatore unico del giorno (usato come chiave in localStorage)
        "nome": "Push",            // * nome visualizzato nel tab

        "esercizi": [
          {
            "id":        "chest_press_mac",             // * ID univoco nell'intera scheda
            "nome":      "Chest Press alla macchina",   // * nome visualizzato
            "serie":     4,                             // * numero intero → genera 4 pallini tracker
            "reps":      "6-8",                         // * stringa libera (es. "6-8", "12", "5-10 min")
            "recupero":  90,                            //   secondi di riposo (intero); default 90 se assente
            "note":      "Machine Chest Press – controllo fase eccentrica"  // * indicazioni tecniche
          }
        ]
      }
    ]
  }
}
```

### Campi esercizio — dettaglio

| Campo | Tipo | Obbligatorio | Descrizione |
|---|---|---|---|
| `id` | stringa | ✅ | Identificatore univoco. Usato come chiave per localStorage e cronologia. Non cambiare dopo l'uso! |
| `nome` | stringa | ✅ | Nome visualizzato nella card esercizio |
| `serie` | intero | ✅ | Numero di serie → genera i pallini cliccabili nel tracker |
| `reps` | stringa | ✅ | Range o valore libero: `"6-8"`, `"12"`, `"8-10 per gamba"`, `"5-10 min"` |
| `recupero` | intero | ❌ | Secondi di riposo tra le serie. Se assente viene usato il default di **90 s**. Usare `60`, `90`, `120`, `180` |
| `note` | stringa | ✅ | Indicazioni tecniche, suggerimenti di esecuzione. Mostrate in giallo sotto i campi input |

> ⚠️ **Attenzione:** l'`id` dell'esercizio è la chiave con cui vengono salvati peso, reps e note in localStorage e nella cronologia. Se cambi l'`id` dopo aver già registrato sessioni, quei dati storici non saranno più visibili in cronologia.

---

## ✏️ Esempio completo — scheda minima funzionante

```json
{
  "scheda": {
    "nome": "ForzeBase",
    "numero": "01",
    "mese": "06",
    "anno": "2026",
    "versione": "1.0",

    "giorni": [
      {
        "id": "full_a",
        "nome": "Full Body A",
        "esercizi": [
          {
            "id":       "squat_bar",
            "nome":     "Squat con bilanciere",
            "serie":    4,
            "reps":     "5",
            "recupero": 180,
            "note":     "Sotto il parallelo, schiena neutra"
          },
          {
            "id":       "panca_bar",
            "nome":     "Panca piana con bilanciere",
            "serie":    4,
            "reps":     "5",
            "recupero": 180,
            "note":     "Scapole strette, piedi piatti a terra"
          },
          {
            "id":       "stacco_bar",
            "nome":     "Stacco da terra",
            "serie":    3,
            "reps":     "5",
            "recupero": 240,
            "note":     "Barre sui metatarsi, schiena rigida"
          }
        ]
      }
    ]
  }
}
```

---

## 🚀 Attivare GitHub Pages

1. Vai in **Settings → Pages**
2. **Source:** `Deploy from a branch`
3. **Branch:** `main` / `/ (root)`
4. Salva — dopo pochi secondi la pagina sarà disponibile su `https://<username>.github.io/<repo>/`

> L'app funziona interamente lato client: nessun server, nessuna build. Basta aprire `index.html`.
