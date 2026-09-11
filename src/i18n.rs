//! Minimal built-in i18n: UI strings for 9 Latin-script languages.
//!
//! Only Latin-script languages are shipped because egui 0.27's embedded
//! font (Ubuntu-Light) was verified to have NO Cyrillic coverage, and no
//! CJK/Arabic coverage either. Adding Russian/Chinese/Arabic requires
//! bundling a custom font first (see roadmap).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Lang {
    En,
    Tr,
    De,
    Fr,
    Es,
    Pt,
    It,
    Nl,
    Pl,
}

impl Default for Lang {
    /// Existing users keep Turkish.
    fn default() -> Self {
        Lang::Tr
    }
}

impl Lang {
    pub fn all() -> &'static [Lang] {
        &[
            Lang::En,
            Lang::Tr,
            Lang::De,
            Lang::Fr,
            Lang::Es,
            Lang::Pt,
            Lang::It,
            Lang::Nl,
            Lang::Pl,
        ]
    }

    pub fn code(&self) -> &'static str {
        match self {
            Lang::En => "en",
            Lang::Tr => "tr",
            Lang::De => "de",
            Lang::Fr => "fr",
            Lang::Es => "es",
            Lang::Pt => "pt",
            Lang::It => "it",
            Lang::Nl => "nl",
            Lang::Pl => "pl",
        }
    }

    /// Native display name for the language picker.
    pub fn label(&self) -> &'static str {
        match self {
            Lang::En => "English",
            Lang::Tr => "Türkçe",
            Lang::De => "Deutsch",
            Lang::Fr => "Français",
            Lang::Es => "Español",
            Lang::Pt => "Português",
            Lang::It => "Italiano",
            Lang::Nl => "Nederlands",
            Lang::Pl => "Polski",
        }
    }

    pub fn from_code(code: &str) -> Option<Lang> {
        Self::all().iter().find(|l| l.code() == code).copied()
    }
}

/// All translatable keys. Tests enforce every key × every language.
#[cfg_attr(not(test), allow(dead_code))]
pub const ALL_KEYS: &[&str] = &[
    "tab_control",
    "tab_macros",
    "tab_settings",
    "tab_about",
    "rec_idle",
    "btn_stop",
    "play_idle",
    "loop_title",
    "loop_infinite",
    "loop_times",
    "loop_apply",
    "file_row",
    "status_line",
    "events_line",
    "macro_line",
    "state_idle",
    "state_recording",
    "state_playing",
    "macros_heading",
    "macros_name",
    "macros_dur",
    "macros_events",
    "macros_created",
    "macros_save",
    "macros_load",
    "macros_fmt1",
    "macros_fmt2",
    "macros_empty",
    "save_quick",
    "load_quick",
    "set_heading",
    "set_record",
    "set_play",
    "set_stop",
    "set_change",
    "set_capture",
    "set_capture2",
    "set_cancel",
    "set_warn",
    "set_reset",
    "set_assigned",
    "set_defaults",
    "set_lang",
    "about_heading",
    "about_sub",
    "about_feat",
    "about_f1",
    "about_f2",
    "about_f3",
    "about_f4",
    "about_f5",
    "about_f6",
    "about_credit",
    "ready",
];

/// Look up a UI string. Falls back to English, then to the key itself
/// (never panics on missing translations).
pub fn t(lang: Lang, key: &'static str) -> &'static str {
    match (lang, key) {
        // ---------- tabs ----------
        (Lang::Tr, "tab_control") => "Kontrol",
        (Lang::Tr, "tab_macros") => "Makrolar",
        (Lang::Tr, "tab_settings") => "Ayarlar",
        (Lang::Tr, "tab_about") => "Hakkında",
        (Lang::En, "tab_control") => "Control",
        (Lang::En, "tab_macros") => "Macros",
        (Lang::En, "tab_settings") => "Settings",
        (Lang::En, "tab_about") => "About",
        (Lang::De, "tab_control") => "Steuerung",
        (Lang::De, "tab_macros") => "Makros",
        (Lang::De, "tab_settings") => "Einstellungen",
        (Lang::De, "tab_about") => "Über",
        (Lang::Fr, "tab_control") => "Contrôle",
        (Lang::Fr, "tab_macros") => "Macros",
        (Lang::Fr, "tab_settings") => "Paramètres",
        (Lang::Fr, "tab_about") => "À propos",
        (Lang::Es, "tab_control") => "Control",
        (Lang::Es, "tab_macros") => "Macros",
        (Lang::Es, "tab_settings") => "Ajustes",
        (Lang::Es, "tab_about") => "Acerca de",
        (Lang::Pt, "tab_control") => "Controlo",
        (Lang::Pt, "tab_macros") => "Macros",
        (Lang::Pt, "tab_settings") => "Definições",
        (Lang::Pt, "tab_about") => "Sobre",
        (Lang::It, "tab_control") => "Controllo",
        (Lang::It, "tab_macros") => "Macro",
        (Lang::It, "tab_settings") => "Impostazioni",
        (Lang::It, "tab_about") => "Info",
        (Lang::Nl, "tab_control") => "Bediening",
        (Lang::Nl, "tab_macros") => "Macro's",
        (Lang::Nl, "tab_settings") => "Instellingen",
        (Lang::Nl, "tab_about") => "Over",
        (Lang::Pl, "tab_control") => "Sterowanie",
        (Lang::Pl, "tab_macros") => "Makra",
        (Lang::Pl, "tab_settings") => "Ustawienia",
        (Lang::Pl, "tab_about") => "O programie",
        // ---------- control buttons ----------
        (Lang::Tr, "rec_idle") => "● Kaydet",
        (Lang::Tr, "btn_stop") => "■ Durdur",
        (Lang::Tr, "play_idle") => "▶ Oynat",
        (Lang::En, "rec_idle") => "● Record",
        (Lang::En, "btn_stop") => "■ Stop",
        (Lang::En, "play_idle") => "▶ Play",
        (Lang::De, "rec_idle") => "● Aufnehmen",
        (Lang::De, "btn_stop") => "■ Stopp",
        (Lang::De, "play_idle") => "▶ Abspielen",
        (Lang::Fr, "rec_idle") => "● Enregistrer",
        (Lang::Fr, "btn_stop") => "■ Arrêter",
        (Lang::Fr, "play_idle") => "▶ Lire",
        (Lang::Es, "rec_idle") => "● Grabar",
        (Lang::Es, "btn_stop") => "■ Detener",
        (Lang::Es, "play_idle") => "▶ Reproducir",
        (Lang::Pt, "rec_idle") => "● Gravar",
        (Lang::Pt, "btn_stop") => "■ Parar",
        (Lang::Pt, "play_idle") => "▶ Reproduzir",
        (Lang::It, "rec_idle") => "● Registra",
        (Lang::It, "btn_stop") => "■ Ferma",
        (Lang::It, "play_idle") => "▶ Riproduci",
        (Lang::Nl, "rec_idle") => "● Opnemen",
        (Lang::Nl, "btn_stop") => "■ Stoppen",
        (Lang::Nl, "play_idle") => "▶ Afspelen",
        (Lang::Pl, "rec_idle") => "● Nagrywaj",
        (Lang::Pl, "btn_stop") => "■ Zatrzymaj",
        (Lang::Pl, "play_idle") => "▶ Odtwórz",
        // ---------- loop settings ----------
        (Lang::Tr, "loop_title") => "Döngü Ayarları:",
        (Lang::Tr, "loop_infinite") => "Sonsuz döngü",
        (Lang::Tr, "loop_times") => "kez",
        (Lang::Tr, "loop_apply") => "Döngü Ayarını Uygula",
        (Lang::En, "loop_title") => "Loop Settings:",
        (Lang::En, "loop_infinite") => "Infinite loop",
        (Lang::En, "loop_times") => "times",
        (Lang::En, "loop_apply") => "Apply Loop Setting",
        (Lang::De, "loop_title") => "Schleifen-Einstellungen:",
        (Lang::De, "loop_infinite") => "Endlosschleife",
        (Lang::De, "loop_times") => "mal",
        (Lang::De, "loop_apply") => "Schleifen-Einstellung übernehmen",
        (Lang::Fr, "loop_title") => "Réglages de boucle :",
        (Lang::Fr, "loop_infinite") => "Boucle infinie",
        (Lang::Fr, "loop_times") => "fois",
        (Lang::Fr, "loop_apply") => "Appliquer le réglage de boucle",
        (Lang::Es, "loop_title") => "Ajustes de bucle:",
        (Lang::Es, "loop_infinite") => "Bucle infinito",
        (Lang::Es, "loop_times") => "veces",
        (Lang::Es, "loop_apply") => "Aplicar ajuste de bucle",
        (Lang::Pt, "loop_title") => "Definições de repetição:",
        (Lang::Pt, "loop_infinite") => "Repetição infinita",
        (Lang::Pt, "loop_times") => "vezes",
        (Lang::Pt, "loop_apply") => "Aplicar definição de repetição",
        (Lang::It, "loop_title") => "Impostazioni ciclo:",
        (Lang::It, "loop_infinite") => "Ciclo infinito",
        (Lang::It, "loop_times") => "volte",
        (Lang::It, "loop_apply") => "Applica impostazione ciclo",
        (Lang::Nl, "loop_title") => "Lus-instellingen:",
        (Lang::Nl, "loop_infinite") => "Oneindige lus",
        (Lang::Nl, "loop_times") => "keer",
        (Lang::Nl, "loop_apply") => "Lus-instelling toepassen",
        (Lang::Pl, "loop_title") => "Ustawienia pętli:",
        (Lang::Pl, "loop_infinite") => "Nieskończona pętla",
        (Lang::Pl, "loop_times") => "razy",
        (Lang::Pl, "loop_apply") => "Zastosuj ustawienie pętli",
        // ---------- status lines ----------
        (Lang::Tr, "file_row") => "Dosya:",
        (Lang::Tr, "status_line") => "Durum: {m}",
        (Lang::Tr, "events_line") => "Kaydedilen event: {n}",
        (Lang::Tr, "macro_line") => "Makro: {name} | Süre: {dur} | Oluşturulma: {ts}",
        (Lang::En, "file_row") => "File:",
        (Lang::En, "status_line") => "Status: {m}",
        (Lang::En, "events_line") => "Recorded events: {n}",
        (Lang::En, "macro_line") => "Macro: {name} | Duration: {dur} | Created: {ts}",
        (Lang::De, "file_row") => "Datei:",
        (Lang::De, "status_line") => "Status: {m}",
        (Lang::De, "events_line") => "Aufgenommene Events: {n}",
        (Lang::De, "macro_line") => "Makro: {name} | Dauer: {dur} | Erstellt: {ts}",
        (Lang::Fr, "file_row") => "Fichier :",
        (Lang::Fr, "status_line") => "État : {m}",
        (Lang::Fr, "events_line") => "Événements enregistrés : {n}",
        (Lang::Fr, "macro_line") => "Macro : {name} | Durée : {dur} | Créée : {ts}",
        (Lang::Es, "file_row") => "Archivo:",
        (Lang::Es, "status_line") => "Estado: {m}",
        (Lang::Es, "events_line") => "Eventos grabados: {n}",
        (Lang::Es, "macro_line") => "Macro: {name} | Duración: {dur} | Creada: {ts}",
        (Lang::Pt, "file_row") => "Ficheiro:",
        (Lang::Pt, "status_line") => "Estado: {m}",
        (Lang::Pt, "events_line") => "Eventos gravados: {n}",
        (Lang::Pt, "macro_line") => "Macro: {name} | Duração: {dur} | Criada: {ts}",
        (Lang::It, "file_row") => "File:",
        (Lang::It, "status_line") => "Stato: {m}",
        (Lang::It, "events_line") => "Eventi registrati: {n}",
        (Lang::It, "macro_line") => "Macro: {name} | Durata: {dur} | Creata: {ts}",
        (Lang::Nl, "file_row") => "Bestand:",
        (Lang::Nl, "status_line") => "Status: {m}",
        (Lang::Nl, "events_line") => "Opgenomen events: {n}",
        (Lang::Nl, "macro_line") => "Macro: {name} | Duur: {dur} | Aangemaakt: {ts}",
        (Lang::Pl, "file_row") => "Plik:",
        (Lang::Pl, "status_line") => "Stan: {m}",
        (Lang::Pl, "events_line") => "Nagrane zdarzenia: {n}",
        (Lang::Pl, "macro_line") => "Makro: {name} | Czas: {dur} | Utworzono: {ts}",
        // ---------- state badges ----------
        (Lang::Tr, "state_idle") => "○ Boşta",
        (Lang::Tr, "state_recording") => "● KAYIT YAPILIYOR",
        (Lang::Tr, "state_playing") => "● OYNATILIYOR",
        (Lang::En, "state_idle") => "○ Idle",
        (Lang::En, "state_recording") => "● RECORDING",
        (Lang::En, "state_playing") => "● PLAYING",
        (Lang::De, "state_idle") => "○ Bereit",
        (Lang::De, "state_recording") => "● AUFNAHME LÄUFT",
        (Lang::De, "state_playing") => "● WIEDERGABE LÄUFT",
        (Lang::Fr, "state_idle") => "○ Inactif",
        (Lang::Fr, "state_recording") => "● ENREGISTREMENT",
        (Lang::Fr, "state_playing") => "● LECTURE",
        (Lang::Es, "state_idle") => "○ Inactivo",
        (Lang::Es, "state_recording") => "● GRABANDO",
        (Lang::Es, "state_playing") => "● REPRODUCIENDO",
        (Lang::Pt, "state_idle") => "○ Inativo",
        (Lang::Pt, "state_recording") => "● A GRAVAR",
        (Lang::Pt, "state_playing") => "● A REPRODUZIR",
        (Lang::It, "state_idle") => "○ Inattivo",
        (Lang::It, "state_recording") => "● REGISTRAZIONE",
        (Lang::It, "state_playing") => "● RIPRODUZIONE",
        (Lang::Nl, "state_idle") => "○ Inactief",
        (Lang::Nl, "state_recording") => "● OPNAME BEZIG",
        (Lang::Nl, "state_playing") => "● AFSPELEN BEZIG",
        (Lang::Pl, "state_idle") => "○ Bezczynny",
        (Lang::Pl, "state_recording") => "● NAGRYWANIE",
        (Lang::Pl, "state_playing") => "● ODTWARZANIE",
        // ---------- macros tab ----------
        (Lang::Tr, "macros_heading") => "Makro Yönetimi",
        (Lang::Tr, "macros_name") => "Ad: {v}",
        (Lang::Tr, "macros_dur") => "Süre: {v}",
        (Lang::Tr, "macros_events") => "Event sayısı: {v}",
        (Lang::Tr, "macros_created") => "Oluşturulma (unix): {v}",
        (Lang::Tr, "macros_save") => "💾 Makroyu Kaydet",
        (Lang::Tr, "macros_load") => "📂 Makro Yükle",
        (Lang::Tr, "macros_fmt1") => "Format: .tts (binary, küçük/hızlı) veya .json (okunabilir/debug).",
        (Lang::Tr, "macros_fmt2") => "Yüklenen makro anında oynatmaya hazır hale gelir.",
        (Lang::Tr, "macros_empty") => {
            "Henüz kayıt yok — önce Kontrol sekmesinde kayıt yapın veya dosya yükleyin."
        }
        (Lang::En, "macros_heading") => "Macro Management",
        (Lang::En, "macros_name") => "Name: {v}",
        (Lang::En, "macros_dur") => "Duration: {v}",
        (Lang::En, "macros_events") => "Event count: {v}",
        (Lang::En, "macros_created") => "Created (unix): {v}",
        (Lang::En, "macros_save") => "💾 Save Macro",
        (Lang::En, "macros_load") => "📂 Load Macro",
        (Lang::En, "macros_fmt1") => "Format: .tts (binary, small/fast) or .json (readable/debug).",
        (Lang::En, "macros_fmt2") => "A loaded macro is instantly ready to play.",
        (Lang::En, "macros_empty") => {
            "No recording yet — record in the Control tab or load a file."
        }
        (Lang::De, "macros_heading") => "Makro-Verwaltung",
        (Lang::De, "macros_name") => "Name: {v}",
        (Lang::De, "macros_dur") => "Dauer: {v}",
        (Lang::De, "macros_events") => "Event-Anzahl: {v}",
        (Lang::De, "macros_created") => "Erstellt (unix): {v}",
        (Lang::De, "macros_save") => "💾 Makro speichern",
        (Lang::De, "macros_load") => "📂 Makro laden",
        (Lang::De, "macros_fmt1") => {
            "Format: .tts (binär, klein/schnell) oder .json (lesbar/debug)."
        }
        (Lang::De, "macros_fmt2") => "Ein geladenes Makro ist sofort abspielbereit.",
        (Lang::De, "macros_empty") => {
            "Noch keine Aufnahme — nimm im Steuerungs-Tab auf oder lade eine Datei."
        }
        (Lang::Fr, "macros_heading") => "Gestion des macros",
        (Lang::Fr, "macros_name") => "Nom : {v}",
        (Lang::Fr, "macros_dur") => "Durée : {v}",
        (Lang::Fr, "macros_events") => "Nombre d'événements : {v}",
        (Lang::Fr, "macros_created") => "Créée (unix) : {v}",
        (Lang::Fr, "macros_save") => "💾 Enregistrer la macro",
        (Lang::Fr, "macros_load") => "📂 Charger une macro",
        (Lang::Fr, "macros_fmt1") => {
            "Format : .tts (binaire, petit/rapide) ou .json (lisible/debug)."
        }
        (Lang::Fr, "macros_fmt2") => "Une macro chargée est immédiatement prête à être lue.",
        (Lang::Fr, "macros_empty") => {
            "Aucun enregistrement — enregistrez dans l'onglet Contrôle ou chargez un fichier."
        }
        (Lang::Es, "macros_heading") => "Gestión de macros",
        (Lang::Es, "macros_name") => "Nombre: {v}",
        (Lang::Es, "macros_dur") => "Duración: {v}",
        (Lang::Es, "macros_events") => "N.º de eventos: {v}",
        (Lang::Es, "macros_created") => "Creada (unix): {v}",
        (Lang::Es, "macros_save") => "💾 Guardar macro",
        (Lang::Es, "macros_load") => "📂 Cargar macro",
        (Lang::Es, "macros_fmt1") => {
            "Formato: .tts (binario, pequeño/rápido) o .json (legible/debug)."
        }
        (Lang::Es, "macros_fmt2") => "Una macro cargada está lista para reproducirse.",
        (Lang::Es, "macros_empty") => {
            "Sin grabación — graba en la pestaña Control o carga un archivo."
        }
        (Lang::Pt, "macros_heading") => "Gestão de macros",
        (Lang::Pt, "macros_name") => "Nome: {v}",
        (Lang::Pt, "macros_dur") => "Duração: {v}",
        (Lang::Pt, "macros_events") => "N.º de eventos: {v}",
        (Lang::Pt, "macros_created") => "Criada (unix): {v}",
        (Lang::Pt, "macros_save") => "💾 Guardar macro",
        (Lang::Pt, "macros_load") => "📂 Carregar macro",
        (Lang::Pt, "macros_fmt1") => {
            "Formato: .tts (binário, pequeno/rápido) ou .json (legível/debug)."
        }
        (Lang::Pt, "macros_fmt2") => "Uma macro carregada está pronta a reproduzir.",
        (Lang::Pt, "macros_empty") => {
            "Sem gravação — grave no separador Controlo ou carregue um ficheiro."
        }
        (Lang::It, "macros_heading") => "Gestione macro",
        (Lang::It, "macros_name") => "Nome: {v}",
        (Lang::It, "macros_dur") => "Durata: {v}",
        (Lang::It, "macros_events") => "Numero eventi: {v}",
        (Lang::It, "macros_created") => "Creata (unix): {v}",
        (Lang::It, "macros_save") => "💾 Salva macro",
        (Lang::It, "macros_load") => "📂 Carica macro",
        (Lang::It, "macros_fmt1") => {
            "Formato: .tts (binario, piccolo/veloce) o .json (leggibile/debug)."
        }
        (Lang::It, "macros_fmt2") => "Una macro caricata è subito pronta.",
        (Lang::It, "macros_empty") => {
            "Nessuna registrazione — registra nella scheda Controllo o carica un file."
        }
        (Lang::Nl, "macros_heading") => "Macrobeheer",
        (Lang::Nl, "macros_name") => "Naam: {v}",
        (Lang::Nl, "macros_dur") => "Duur: {v}",
        (Lang::Nl, "macros_events") => "Aantal events: {v}",
        (Lang::Nl, "macros_created") => "Aangemaakt (unix): {v}",
        (Lang::Nl, "macros_save") => "💾 Macro opslaan",
        (Lang::Nl, "macros_load") => "📂 Macro laden",
        (Lang::Nl, "macros_fmt1") => {
            "Formaat: .tts (binair, klein/snel) of .json (leesbaar/debug)."
        }
        (Lang::Nl, "macros_fmt2") => "Een geladen macro is direct afspeelbaar.",
        (Lang::Nl, "macros_empty") => {
            "Nog geen opname — neem op via Bediening of laad een bestand."
        }
        (Lang::Pl, "macros_heading") => "Zarządzanie makrami",
        (Lang::Pl, "macros_name") => "Nazwa: {v}",
        (Lang::Pl, "macros_dur") => "Czas: {v}",
        (Lang::Pl, "macros_events") => "Liczba zdarzeń: {v}",
        (Lang::Pl, "macros_created") => "Utworzono (unix): {v}",
        (Lang::Pl, "macros_save") => "💾 Zapisz makro",
        (Lang::Pl, "macros_load") => "📂 Wczytaj makro",
        (Lang::Pl, "macros_fmt1") => {
            "Format: .tts (binarny, mały/szybki) lub .json (czytelny/debug)."
        }
        (Lang::Pl, "macros_fmt2") => "Wczytane makro jest od razu gotowe do odtworzenia.",
        (Lang::Pl, "macros_empty") => {
            "Brak nagrania — nagraj w zakładce Sterowanie lub wczytaj plik."
        }
        // ---------- quick save/load ----------
        (Lang::Tr, "save_quick") => "💾 Kaydet",
        (Lang::Tr, "load_quick") => "📂 Yükle",
        (Lang::En, "save_quick") => "💾 Save",
        (Lang::En, "load_quick") => "📂 Load",
        (Lang::De, "save_quick") => "💾 Speichern",
        (Lang::De, "load_quick") => "📂 Laden",
        (Lang::Fr, "save_quick") => "💾 Enregistrer",
        (Lang::Fr, "load_quick") => "📂 Charger",
        (Lang::Es, "save_quick") => "💾 Guardar",
        (Lang::Es, "load_quick") => "📂 Cargar",
        (Lang::Pt, "save_quick") => "💾 Guardar",
        (Lang::Pt, "load_quick") => "📂 Carregar",
        (Lang::It, "save_quick") => "💾 Salva",
        (Lang::It, "load_quick") => "📂 Carica",
        (Lang::Nl, "save_quick") => "💾 Opslaan",
        (Lang::Nl, "load_quick") => "📂 Laden",
        (Lang::Pl, "save_quick") => "💾 Zapisz",
        (Lang::Pl, "load_quick") => "📂 Wczytaj",
        // ---------- settings tab ----------
        (Lang::Tr, "set_heading") => "Klavye Kısayolları",
        (Lang::Tr, "set_record") => "Kayıt Başlat/Durdur:",
        (Lang::Tr, "set_play") => "Oynat Başlat:",
        (Lang::Tr, "set_stop") => "Oynatmayı Durdur:",
        (Lang::Tr, "set_change") => "Değiştir",
        (Lang::Tr, "set_capture") => {
            "Yeni kısayolu girin: tek tuş (örn. F8) veya Ctrl/Alt/Shift ile birlikte bir tuş..."
        }
        (Lang::Tr, "set_capture2") => "Atamak için tuşa basın. Vazgeçmek için İptal'e tıklayın.",
        (Lang::Tr, "set_cancel") => "İptal",
        (Lang::Tr, "set_warn") => {
            "⚠ Tek tuş (örn. A, Space) yazı yazarken de tetiklenir. F8–F12 önerilir."
        }
        (Lang::Tr, "set_reset") => "Varsayılan Kısayollara Döndür",
        (Lang::Tr, "set_assigned") => "Atandı: {v}",
        (Lang::Tr, "set_defaults") => "Varsayılanlara dönüldü.",
        (Lang::Tr, "set_lang") => "Dil:",
        (Lang::En, "set_heading") => "Keyboard Shortcuts",
        (Lang::En, "set_record") => "Start/Stop recording:",
        (Lang::En, "set_play") => "Start playback:",
        (Lang::En, "set_stop") => "Stop playback:",
        (Lang::En, "set_change") => "Change",
        (Lang::En, "set_capture") => {
            "Enter the new shortcut: a single key (e.g. F8) or a key with Ctrl/Alt/Shift..."
        }
        (Lang::En, "set_capture2") => "Press a key to assign it. Click Cancel to abort.",
        (Lang::En, "set_cancel") => "Cancel",
        (Lang::En, "set_warn") => {
            "⚠ Single keys (e.g. A, Space) also fire while typing. F8–F12 recommended."
        }
        (Lang::En, "set_reset") => "Reset to Defaults",
        (Lang::En, "set_assigned") => "Assigned: {v}",
        (Lang::En, "set_defaults") => "Reset to defaults.",
        (Lang::En, "set_lang") => "Language:",
        (Lang::De, "set_heading") => "Tastaturkürzel",
        (Lang::De, "set_record") => "Aufnahme starten/stoppen:",
        (Lang::De, "set_play") => "Wiedergabe starten:",
        (Lang::De, "set_stop") => "Wiedergabe stoppen:",
        (Lang::De, "set_change") => "Ändern",
        (Lang::De, "set_capture") => {
            "Neues Kürzel eingeben: einzelne Taste (z. B. F8) oder Taste mit Strg/Alt/Umschalt..."
        }
        (Lang::De, "set_capture2") => "Taste zum Zuweisen drücken. Zum Abbrechen auf Abbrechen klicken.",
        (Lang::De, "set_cancel") => "Abbrechen",
        (Lang::De, "set_warn") => {
            "⚠ Einzeltasten (z. B. A, Leertaste) lösen auch beim Tippen aus. F8–F12 empfohlen."
        }
        (Lang::De, "set_reset") => "Auf Standard zurücksetzen",
        (Lang::De, "set_assigned") => "Zugewiesen: {v}",
        (Lang::De, "set_defaults") => "Auf Standard zurückgesetzt.",
        (Lang::De, "set_lang") => "Sprache:",
        (Lang::Fr, "set_heading") => "Raccourcis clavier",
        (Lang::Fr, "set_record") => "Démarrer/arrêter l'enregistrement :",
        (Lang::Fr, "set_play") => "Démarrer la lecture :",
        (Lang::Fr, "set_stop") => "Arrêter la lecture :",
        (Lang::Fr, "set_change") => "Modifier",
        (Lang::Fr, "set_capture") => {
            "Saisissez le nouveau raccourci : une seule touche (ex. F8) ou une touche avec Ctrl/Alt/Maj..."
        }
        (Lang::Fr, "set_capture2") => {
            "Appuyez sur une touche pour l'assigner. Cliquez sur Annuler pour abandonner."
        }
        (Lang::Fr, "set_cancel") => "Annuler",
        (Lang::Fr, "set_warn") => {
            "⚠ Les touches seules (ex. A, Espace) se déclenchent aussi en tapant. F8–F12 recommandés."
        }
        (Lang::Fr, "set_reset") => "Rétablir les défauts",
        (Lang::Fr, "set_assigned") => "Assigné : {v}",
        (Lang::Fr, "set_defaults") => "Valeurs par défaut rétablies.",
        (Lang::Fr, "set_lang") => "Langue :",
        (Lang::Es, "set_heading") => "Atajos de teclado",
        (Lang::Es, "set_record") => "Iniciar/detener grabación:",
        (Lang::Es, "set_play") => "Iniciar reproducción:",
        (Lang::Es, "set_stop") => "Detener reproducción:",
        (Lang::Es, "set_change") => "Cambiar",
        (Lang::Es, "set_capture") => {
            "Introduce el nuevo atajo: una sola tecla (p. ej. F8) o una tecla con Ctrl/Alt/Mayús..."
        }
        (Lang::Es, "set_capture2") => "Pulsa una tecla para asignarla. Clic en Cancelar para abortar.",
        (Lang::Es, "set_cancel") => "Cancelar",
        (Lang::Es, "set_warn") => {
            "⚠ Las teclas solas (p. ej. A, Espacio) también se activan al escribir. Se recomienda F8–F12."
        }
        (Lang::Es, "set_reset") => "Restablecer valores",
        (Lang::Es, "set_assigned") => "Asignado: {v}",
        (Lang::Es, "set_defaults") => "Valores restablecidos.",
        (Lang::Es, "set_lang") => "Idioma:",
        (Lang::Pt, "set_heading") => "Atalhos de teclado",
        (Lang::Pt, "set_record") => "Iniciar/parar gravação:",
        (Lang::Pt, "set_play") => "Iniciar reprodução:",
        (Lang::Pt, "set_stop") => "Parar reprodução:",
        (Lang::Pt, "set_change") => "Alterar",
        (Lang::Pt, "set_capture") => {
            "Introduza o novo atalho: uma só tecla (ex. F8) ou tecla com Ctrl/Alt/Shift..."
        }
        (Lang::Pt, "set_capture2") => "Prima uma tecla para atribuir. Clique em Cancelar para desistir.",
        (Lang::Pt, "set_cancel") => "Cancelar",
        (Lang::Pt, "set_warn") => {
            "⚠ Teclas isoladas (ex. A, Espaço) disparam também ao escrever. Recomenda-se F8–F12."
        }
        (Lang::Pt, "set_reset") => "Repor predefinições",
        (Lang::Pt, "set_assigned") => "Atribuído: {v}",
        (Lang::Pt, "set_defaults") => "Predefinições repostas.",
        (Lang::Pt, "set_lang") => "Idioma:",
        (Lang::It, "set_heading") => "Scorciatoie da tastiera",
        (Lang::It, "set_record") => "Avvia/ferma registrazione:",
        (Lang::It, "set_play") => "Avvia riproduzione:",
        (Lang::It, "set_stop") => "Ferma riproduzione:",
        (Lang::It, "set_change") => "Cambia",
        (Lang::It, "set_capture") => {
            "Inserisci la nuova scorciatoia: un singolo tasto (es. F8) o un tasto con Ctrl/Alt/Maiusc..."
        }
        (Lang::It, "set_capture2") => "Premi un tasto per assegnarlo. Fai clic su Annulla per uscire.",
        (Lang::It, "set_cancel") => "Annulla",
        (Lang::It, "set_warn") => {
            "⚠ I tasti singoli (es. A, Spazio) si attivano anche digitando. Consigliati F8–F12."
        }
        (Lang::It, "set_reset") => "Ripristina predefinite",
        (Lang::It, "set_assigned") => "Assegnato: {v}",
        (Lang::It, "set_defaults") => "Predefinite ripristinate.",
        (Lang::It, "set_lang") => "Lingua:",
        (Lang::Nl, "set_heading") => "Sneltoetsen",
        (Lang::Nl, "set_record") => "Opname starten/stoppen:",
        (Lang::Nl, "set_play") => "Afspelen starten:",
        (Lang::Nl, "set_stop") => "Afspelen stoppen:",
        (Lang::Nl, "set_change") => "Wijzigen",
        (Lang::Nl, "set_capture") => {
            "Voer de nieuwe sneltoets in: één toets (bijv. F8) of een toets met Ctrl/Alt/Shift..."
        }
        (Lang::Nl, "set_capture2") => "Druk op een toets om toe te wijzen. Klik op Annuleren om af te breken.",
        (Lang::Nl, "set_cancel") => "Annuleren",
        (Lang::Nl, "set_warn") => {
            "⚠ Losse toetsen (bijv. A, Spatie) werken ook tijdens typen. F8–F12 aanbevolen."
        }
        (Lang::Nl, "set_reset") => "Standaard herstellen",
        (Lang::Nl, "set_assigned") => "Toegewezen: {v}",
        (Lang::Nl, "set_defaults") => "Standaard hersteld.",
        (Lang::Nl, "set_lang") => "Taal:",
        (Lang::Pl, "set_heading") => "Skróty klawiszowe",
        (Lang::Pl, "set_record") => "Start/stop nagrywania:",
        (Lang::Pl, "set_play") => "Start odtwarzania:",
        (Lang::Pl, "set_stop") => "Stop odtwarzania:",
        (Lang::Pl, "set_change") => "Zmień",
        (Lang::Pl, "set_capture") => {
            "Wprowadź nowy skrót: pojedynczy klawisz (np. F8) lub klawisz z Ctrl/Alt/Shift..."
        }
        (Lang::Pl, "set_capture2") => "Naciśnij klawisz, aby przypisać. Kliknij Anuluj, aby przerwać.",
        (Lang::Pl, "set_cancel") => "Anuluj",
        (Lang::Pl, "set_warn") => {
            "⚠ Pojedyncze klawisze (np. A, Spacja) zadziałają też podczas pisania. Zalecane F8–F12."
        }
        (Lang::Pl, "set_reset") => "Przywróć domyślne",
        (Lang::Pl, "set_assigned") => "Przypisano: {v}",
        (Lang::Pl, "set_defaults") => "Przywrócono domyślne.",
        (Lang::Pl, "set_lang") => "Język:",
        // ---------- about tab ----------
        (Lang::Tr, "about_heading") => "Hakkında",
        (Lang::Tr, "about_sub") => "Minimalist makro kaydedici ve oynatıcı",
        (Lang::Tr, "about_feat") => "Özellikler:",
        (Lang::Tr, "about_f1") => "• Kernel seviyesinde girdi yakalama",
        (Lang::Tr, "about_f2") => "• X11/Wayland bağımsız",
        (Lang::Tr, "about_f3") => "• Milisaniye hassasiyeti",
        (Lang::Tr, "about_f4") => "• Oyun desteği",
        (Lang::Tr, "about_f5") => "• Döngü modu",
        (Lang::Tr, "about_f6") => "• Özelleştirilebilir kısayollar",
        (Lang::Tr, "about_credit") => "Luna tarafından geliştirilmiştir.",
        (Lang::En, "about_heading") => "About",
        (Lang::En, "about_sub") => "Minimalist macro recorder and player",
        (Lang::En, "about_feat") => "Features:",
        (Lang::En, "about_f1") => "• Kernel-level input capture",
        (Lang::En, "about_f2") => "• X11/Wayland independent",
        (Lang::En, "about_f3") => "• Millisecond precision",
        (Lang::En, "about_f4") => "• Game support",
        (Lang::En, "about_f5") => "• Loop mode",
        (Lang::En, "about_f6") => "• Customizable shortcuts",
        (Lang::En, "about_credit") => "Developed by Luna.",
        (Lang::De, "about_heading") => "Über",
        (Lang::De, "about_sub") => "Minimalistischer Makro-Rekorder und -Player",
        (Lang::De, "about_feat") => "Funktionen:",
        (Lang::De, "about_f1") => "• Input-Erfassung auf Kernel-Ebene",
        (Lang::De, "about_f2") => "• X11/Wayland-unabhängig",
        (Lang::De, "about_f3") => "• Millisekunden-Präzision",
        (Lang::De, "about_f4") => "• Spiele-Unterstützung",
        (Lang::De, "about_f5") => "• Schleifen-Modus",
        (Lang::De, "about_f6") => "• Anpassbare Kürzel",
        (Lang::De, "about_credit") => "Entwickelt von Luna.",
        (Lang::Fr, "about_heading") => "À propos",
        (Lang::Fr, "about_sub") => "Enregistreur et lecteur de macros minimaliste",
        (Lang::Fr, "about_feat") => "Fonctionnalités :",
        (Lang::Fr, "about_f1") => "• Capture d'entrée au niveau du noyau",
        (Lang::Fr, "about_f2") => "• Indépendant de X11/Wayland",
        (Lang::Fr, "about_f3") => "• Précision à la milliseconde",
        (Lang::Fr, "about_f4") => "• Prise en charge des jeux",
        (Lang::Fr, "about_f5") => "• Mode boucle",
        (Lang::Fr, "about_f6") => "• Raccourcis personnalisables",
        (Lang::Fr, "about_credit") => "Développé par Luna.",
        (Lang::Es, "about_heading") => "Acerca de",
        (Lang::Es, "about_sub") => "Grabador y reproductor de macros minimalista",
        (Lang::Es, "about_feat") => "Características:",
        (Lang::Es, "about_f1") => "• Captura de entrada a nivel de kernel",
        (Lang::Es, "about_f2") => "• Independiente de X11/Wayland",
        (Lang::Es, "about_f3") => "• Precisión de milisegundos",
        (Lang::Es, "about_f4") => "• Soporte para juegos",
        (Lang::Es, "about_f5") => "• Modo bucle",
        (Lang::Es, "about_f6") => "• Atajos personalizables",
        (Lang::Es, "about_credit") => "Desarrollado por Luna.",
        (Lang::Pt, "about_heading") => "Sobre",
        (Lang::Pt, "about_sub") => "Gravador e reprodutor de macros minimalista",
        (Lang::Pt, "about_feat") => "Funcionalidades:",
        (Lang::Pt, "about_f1") => "• Captura de entrada ao nível do kernel",
        (Lang::Pt, "about_f2") => "• Independente de X11/Wayland",
        (Lang::Pt, "about_f3") => "• Precisão de milissegundos",
        (Lang::Pt, "about_f4") => "• Suporte a jogos",
        (Lang::Pt, "about_f5") => "• Modo de repetição",
        (Lang::Pt, "about_f6") => "• Atalhos personalizáveis",
        (Lang::Pt, "about_credit") => "Desenvolvido por Luna.",
        (Lang::It, "about_heading") => "Info",
        (Lang::It, "about_sub") => "Registratore e riproduttore di macro minimalista",
        (Lang::It, "about_feat") => "Funzionalità:",
        (Lang::It, "about_f1") => "• Cattura input a livello kernel",
        (Lang::It, "about_f2") => "• Indipendente da X11/Wayland",
        (Lang::It, "about_f3") => "• Precisione al millisecondo",
        (Lang::It, "about_f4") => "• Supporto giochi",
        (Lang::It, "about_f5") => "• Modalità ciclo",
        (Lang::It, "about_f6") => "• Scorciatoie personalizzabili",
        (Lang::It, "about_credit") => "Sviluppato da Luna.",
        (Lang::Nl, "about_heading") => "Over",
        (Lang::Nl, "about_sub") => "Minimalistische macro-opnemer en -speler",
        (Lang::Nl, "about_feat") => "Functies:",
        (Lang::Nl, "about_f1") => "• Input-opname op kernelniveau",
        (Lang::Nl, "about_f2") => "• X11/Wayland-onafhankelijk",
        (Lang::Nl, "about_f3") => "• Millisecondeprecisie",
        (Lang::Nl, "about_f4") => "• Game-ondersteuning",
        (Lang::Nl, "about_f5") => "• Lusmodus",
        (Lang::Nl, "about_f6") => "• Aanpasbare sneltoetsen",
        (Lang::Nl, "about_credit") => "Ontwikkeld door Luna.",
        (Lang::Pl, "about_heading") => "O programie",
        (Lang::Pl, "about_sub") => "Minimalistyczny rejestrator i odtwarzacz makr",
        (Lang::Pl, "about_feat") => "Funkcje:",
        (Lang::Pl, "about_f1") => "• Przechwytywanie wejścia na poziomie jądra",
        (Lang::Pl, "about_f2") => "• Niezależność od X11/Wayland",
        (Lang::Pl, "about_f3") => "• Precyzja milisekundowa",
        (Lang::Pl, "about_f4") => "• Wsparcie gier",
        (Lang::Pl, "about_f5") => "• Tryb pętli",
        (Lang::Pl, "about_f6") => "• Konfigurowalne skróty",
        (Lang::Pl, "about_credit") => "Autor: Luna.",
        // ---------- misc ----------
        (Lang::Tr, "ready") => "Hazır",
        (Lang::En, "ready") => "Ready",
        (Lang::De, "ready") => "Bereit",
        (Lang::Fr, "ready") => "Prêt",
        (Lang::Es, "ready") => "Listo",
        (Lang::Pt, "ready") => "Pronto",
        (Lang::It, "ready") => "Pronto",
        (Lang::Nl, "ready") => "Gereed",
        (Lang::Pl, "ready") => "Gotowy",
        // ---------- fallback: English for anything not yet translated ----------
        (_, key) => match key {
            "tab_control" => "Control",
            "tab_macros" => "Macros",
            "tab_settings" => "Settings",
            "tab_about" => "About",
            "ready" => "Ready",
            _ => key,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_key_translated_in_every_language() {
        for lang in Lang::all() {
            for key in ALL_KEYS {
                let s = t(*lang, *key);
                assert!(
                    !s.is_empty() && s != *key,
                    "missing translation: lang={:?} key={}",
                    lang,
                    key
                );
            }
        }
    }

    #[test]
    fn templates_keep_placeholders() {
        // Templates with placeholders must survive translation in every language.
        for lang in Lang::all() {
            assert!(
                t(*lang, "status_line").contains("{m}"),
                "status_line placeholder lost in {:?}",
                lang
            );
            assert!(
                t(*lang, "macro_line").contains("{name}"),
                "macro_line placeholder lost in {:?}",
                lang
            );
            assert!(
                t(*lang, "set_assigned").contains("{v}"),
                "set_assigned placeholder lost in {:?}",
                lang
            );
        }
    }

    #[test]
    fn from_code_roundtrip_and_default() {
        assert_eq!(Lang::default(), Lang::Tr);
        for lang in Lang::all() {
            assert_eq!(Lang::from_code(lang.code()), Some(*lang));
        }
        assert_eq!(Lang::from_code("xx"), None);
    }

    #[test]
    fn unknown_key_falls_back() {
        assert_eq!(t(Lang::De, "tab_control"), "Steuerung");
        assert_eq!(t(Lang::De, "no_such_key"), "no_such_key");
    }
}
