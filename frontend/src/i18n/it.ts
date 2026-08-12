/**
 * Italian copy.
 *
 * The `chip.*` and `signal.*` keys are the ones the Rust side names through
 * `explain_key`. They carry the plain-language explanations the Radar shows to
 * someone who has never heard of a rolling code, so they are written for that
 * reader: no jargon without an immediate gloss, and concrete everyday examples
 * instead of abstractions.
 */
export const it = {
  "app.name": "FlipperZero Tool",

  "nav.radar": "Radar",
  "nav.files": "File",
  "nav.library": "Libreria",
  "nav.device": "Dispositivo",
  "nav.settings": "Impostazioni",
  "nav.tools": "Strumenti",
  "nav.sections": "Sezioni",
  "files.mode.local": "Locale",
  "files.mode.serial": "Flipper",
  "files.up": "Su",
  "files.no_folder": "(nessuna cartella selezionata)",
  "files.search": "Cerca...",
  "files.new": "+ Nuovo",
  "files.loading": "Caricamento...",
  "files.empty": "Cartella vuota",
  "files.no_match": "Nessun file corrisponde alla ricerca",
  "files.connect_first": "Collega il Flipper per sfogliarne i file",
  "device.disconnected": "Disconnesso",
  "tools.analyze": "Analisi dei dati grezzi",
  "tools.compare": "Confronto catture",
  "tools.compare.left": "File a sinistra",
  "tools.compare.right": "File a destra",
  "tools.compare.needs_two": "Apri almeno due file nell'editor per confrontarli.",
  "tools.build": "Sviluppo app (.fap)",
  "tools.build.desktop_only": "La compilazione dei .fap richiede uFBT, che gira solo su desktop. Questa sezione resta visibile per chiarezza, ma qui non è utilizzabile.",
  "device.subtitle": "Scegli come collegarti al Flipper.",
  "settings.language": "Lingua",
  "settings.mock": "Dispositivo simulato",
  "settings.mock.on": "Simulazione attiva",
  "settings.mock.off": "Simulazione disattivata",
  "settings.mock.hint": "Mostra segnali di esempio senza un Flipper collegato, utile per esplorare l'app.",

  "radar.title": "Cosa vede il Flipper",
  "radar.subtitle": "Tocca un chip per vedere cosa sta captando",
  "onboarding.what.title": "Questo è ciò che il Flipper sente",
  "onboarding.what.body": "Il Flipper ha diverse radio, ognuna sensibile a un tipo di segnale diverso: telecomandi di cancelli, carte contactless, badge, telecomandi TV. Lo schema mostra tutte le radio: tocca quella che ti incuriosisce.",
  "onboarding.counts.title": "I numeri contano cose diverse",
  "onboarding.counts.body": "Il numero su un chip dice quanti segnali *diversi* ha visto, non quante volte. Se passi dieci volte davanti allo stesso cancello resta un segnale solo, con il contatore degli avvistamenti che sale.",
  "onboarding.limits.title": "Cosa puoi fare, e cosa no",
  "onboarding.limits.body": "Osservare e analizzare è sempre lecito. Ritrasmettere o emulare un segnale è un'altra cosa: fallo solo su dispositivi tuoi o per cui hai un permesso esplicito. In gran parte dei paesi, Italia compresa, farlo su apparecchi altrui è illegale.",
  "onboarding.next": "Avanti",
  "onboarding.skip": "Salta",
  "onboarding.start": "Iniziamo",
  "radar.empty": "Nessun segnale rilevato per ora",
  "radar.empty.hint":
    "Avvicina il Flipper a un telecomando, una carta o un lettore. I segnali compaiono qui appena vengono captati.",
  "radar.disconnected": "Flipper non connesso",
  "radar.disconnected.hint": "Collega il Flipper per iniziare a vedere i segnali intorno a te.",
  "radar.signals.count_one": "{count} segnale",
  "radar.signals.count_other": "{count} segnali",
  "radar.chip.unavailable": "Non disponibile in questa modalità",
  "radar.chip.unavailable.ble":
    "La radio Bluetooth del Flipper è occupata a parlare con questo telefono, quindi non può contemporaneamente cercare altri dispositivi Bluetooth intorno. Collega il Flipper via cavo USB per usare questa funzione.",
  "radar.chip.not_detected": "Modulo non collegato",
  "radar.chip.not_detected.hint":
    "Questo modulo si aggiunge al Flipper tramite i pin GPIO. Compare qui appena viene rilevato.",

  "signal.seen_count_one": "Visto {count} volta",
  "signal.seen_count_other": "Visto {count} volte",
  "signal.first_seen": "Primo avvistamento",
  "signal.last_seen": "Ultimo avvistamento",
  "signal.strength": "Potenza del segnale",
  "signal.strength.hint":
    "Quanto arriva forte. Più il numero è vicino a zero, più la sorgente è vicina.",
  "signal.raw": "Dati grezzi",

  // What each chip is, for someone who knows nothing about radio.
  "chip.subghz": "Radio a lunga distanza",
  "chip.subghz.detail":
    "Ascolta le radio a bassa frequenza usate da telecomandi di cancelli e garage, campanelli, sensori di temperatura e stazioni meteo. È il chip che capta più cose all'aperto.",
  "chip.nfc": "Lettura carte a contatto",
  "chip.nfc.detail":
    "Legge le carte che avvicini a un lettore: badge dell'ufficio, abbonamenti dei mezzi, carte contactless. Funziona solo a pochi centimetri di distanza.",
  "chip.lfrfid": "Badge a bassa frequenza",
  "chip.lfrfid.detail":
    "Legge i badge più vecchi e semplici, tipici di portoni condominiali e cancelli aziendali. Contengono di solito solo un numero di identificazione.",
  "chip.ibutton": "Chiavi a contatto metallico",
  "chip.ibutton.detail":
    "Le chiavette tonde di metallo che si appoggiano a un lettore, comuni sui citofoni italiani. Vanno toccate fisicamente per essere lette.",
  "chip.infrared": "Telecomandi a infrarossi",
  "chip.infrared.detail":
    "La luce invisibile che usano i telecomandi di TV, condizionatori e decoder. Serve puntare il Flipper verso l'apparecchio, senza ostacoli in mezzo.",
  "chip.bluetooth": "Dispositivi Bluetooth vicini",
  "chip.bluetooth.detail":
    "Cuffie, smartwatch, tracker e altoparlanti che annunciano la propria presenza nelle vicinanze.",
  "chip.gpio": "Pin di collegamento",
  "chip.gpio.detail":
    "I contatti sul dorso del Flipper a cui si collegano moduli aggiuntivi o circuiti elettronici.",
  "chip.wifi_devboard": "Modulo WiFi",
  "chip.wifi_devboard.detail":
    "Scheda aggiuntiva basata su ESP32 che aggiunge al Flipper le funzioni WiFi, che da solo non ha.",
  "chip.external_cc1101": "Radio esterna",
  "chip.external_cc1101.detail":
    "Una seconda antenna radio collegata esternamente, con portata maggiore di quella integrata.",
  "chip.nrf24": "Modulo NRF24",
  "chip.nrf24.detail":
    "Modulo radio a 2.4 GHz, usato da alcune tastiere e mouse senza fili.",
  "chip.unknown_module": "Modulo sconosciuto",
  "chip.unknown_module.detail":
    "Qualcosa è collegato ai pin GPIO ma non siamo riusciti a riconoscerlo.",

  // What a specific signal is.
  "signal.subghz.rolling": "Telecomando a codice variabile",
  "signal.subghz.rolling.detail":
    "Questo telecomando cambia codice a ogni pressione, apposta per non poter essere copiato. Puoi vederlo e analizzarlo, ma ritrasmetterlo non aprirebbe nulla: il ricevitore ha già scartato quel codice.",
  "signal.subghz.fixed": "Telecomando a codice fisso",
  "signal.subghz.fixed.detail":
    "Manda sempre lo stesso codice a ogni pressione. È il tipo più semplice e più vecchio, usato da molti cancelli e garage.",
  "signal.subghz.unknown": "Segnale radio",
  "signal.subghz.unknown.detail":
    "Abbiamo captato una trasmissione ma non siamo ancora riusciti a capire di che tipo di dispositivo si tratti.",
  "signal.nfc": "Carta contactless",
  "signal.nfc.detail":
    "Ogni carta ha un numero di serie univoco, chiamato UID. Alcune contengono anche dati protetti che non sempre si riescono a leggere.",
  "signal.lfrfid": "Badge 125 kHz",
  "signal.lfrfid.detail": "Un badge semplice che contiene un numero di identificazione fisso.",
  "signal.ibutton": "Chiave iButton",
  "signal.ibutton.detail": "Una chiavetta metallica con un numero di serie univoco inciso dentro.",
  "signal.infrared": "Comando a infrarossi",
  "signal.infrared.detail":
    "Un singolo tasto di un telecomando, ad esempio accensione o cambio canale.",
  "signal.bluetooth": "Dispositivo Bluetooth",
  "signal.bluetooth.detail":
    "Un apparecchio che sta annunciando la propria presenza. Non significa che sia connesso a te.",
  "signal.gpio": "Stato di un pin",
  "signal.gpio.detail": "Il livello elettrico letto su uno dei contatti del Flipper.",
  "signal.module": "Modulo collegato",
  "signal.module.detail": "Una scheda aggiuntiva che si è annunciata sui pin GPIO.",

  // Actions.
  "action.save": "Salva",
  "action.save.detail": "Conserva questa cattura sulla scheda SD del Flipper.",
  "action.replay": "Ritrasmetti",
  "action.replay.detail": "Il Flipper rimanda in aria esattamente questo segnale.",
  "action.emulate": "Emula",
  "action.emulate.detail": "Il Flipper si finge questa carta o chiave davanti a un lettore.",
  "action.analyze": "Analizza",
  "action.analyze.detail": "Esamina i dati grezzi per capire com'è fatto il segnale.",
  "action.compare": "Confronta",
  "action.compare.detail": "Mette a confronto questa cattura con un'altra per vedere cosa cambia.",
  "action.export": "Esporta",
  "action.export.detail": "Salva la cattura sul telefono o condividila.",
  "firmware.official": "Firmware ufficiale",
  "firmware.momentum": "Momentum (firmware custom)",
  "firmware.unleashed": "Unleashed (firmware custom)",
  "firmware.roguemaster": "RogueMaster (firmware custom)",
  "firmware.unknown": "Firmware non riconosciuto",
  "action.needs_fap": "Serve l'app Signal Radar sul Flipper",
  "action.tx_not_implemented": "L'app sul dispositivo non sa ancora trasmettere",
  "action.frequency_blocked": "Questo firmware non trasmette su questa frequenza",
  "action.compare.pick_second": "Scegli una seconda cattura da confrontare",

  // The legal gate. Deliberately plain and non-negotiable in tone.
  "legal.title": "Prima di trasmettere",
  "legal.body":
    "Stai per far trasmettere qualcosa al Flipper. Fallo solo su dispositivi tuoi, o per cui hai un permesso esplicito. Trasmettere verso apparecchi altrui è illegale in gran parte dei paesi, Italia compresa, anche se lo fai per curiosità.",
  "legal.confirm": "Confermo che è mio o ho il permesso",
  "legal.cancel": "Annulla",

  "common.cancel": "Annulla",
  "common.close": "Chiudi",
  "common.retry": "Riprova",
  "common.unknown": "Sconosciuto",
  "common.never": "Mai",
} as const;
