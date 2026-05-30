# Piattaforma Allenamento Estremo

Web app statica per GitHub Pages. Le schede vengono caricate dalla cartella `schede/` e i progressi giornalieri vengono salvati in `localStorage`.

## Convenzione nomi file

```
schede/NOME_NUMERO_mese-anno.json
```

Esempio: `schede/ForzeBase_01_01-2026.json`

Dopo aver aggiunto una scheda, aggiorna anche `schede/catalog.json`.

## Struttura JSON

```json
{
  "scheda": {
    "nome": "NomeScheda",
    "numero": "01",
    "mese": "01",
    "anno": "2026",
    "giorni": [
      { "id": "lun", "nome": "Push", "esercizi": [ ... ] }
    ]
  }
}
```

## GitHub Pages

Attiva da **Settings → Pages → Source: main / root**.
