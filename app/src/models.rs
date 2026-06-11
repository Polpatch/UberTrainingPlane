use gloo_storage::{LocalStorage, Storage};
use js_sys::Date as JsDate;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

// ── Core workout data ────────────────────────────────────────────────────────

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Workout {
    pub id: String,
    pub nome: String,
    pub descrizione: Option<String>,
    pub categoria: Option<String>,
    pub giorni: Vec<Day>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Day {
    pub giorno: String,
    pub etichetta: Option<String>,
    pub esercizi: Vec<Exercise>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Exercise {
    pub id: String,
    pub nome: String,
    pub serie: u32,
    pub reps: String,
    pub recupero: Option<u32>,
    pub note: Option<String>,
    pub video: Option<String>,
    #[serde(default)]
    pub tipo: Option<String>,
    #[serde(default)]
    pub durata: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct CatalogEntry {
    pub file: String,
    pub nome: String,
    pub numero: Option<String>,
    pub mese: Option<String>,
    pub anno: Option<String>,
    pub preferita: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct CompletedSet {
    pub exercise_id: String,
    pub nome: String,
    pub set_number: u32,
    pub peso: Option<f32>,
    pub reps: Option<String>,
    pub timestamp: String,
    #[serde(default)]
    pub durata_min: Option<u32>,
}

// ── Session schema ───────────────────────────────────────────────────────────

/// Full session stored under `sessions__{workout_id}`.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Session {
    pub id: String,
    pub workout_id: String,
    pub workout_nome: String,
    pub day: String,
    pub started: String,
    pub updated: String,
    pub done: bool,
    pub active_exercise: usize,
    pub sets: Vec<CompletedSet>,
}

/// Lightweight entry stored in the `sessions_index` key.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct SessionMeta {
    pub id: String,
    pub workout_id: String,
    pub workout_nome: String,
    pub day: String,
    pub started: String,
    pub updated: String,
    pub done: bool,
    pub completion_pct: f32,
}

// ── Timestamp helpers ────────────────────────────────────────────────────────

pub fn now_iso() -> String {
    JsDate::new_0().to_iso_string().as_string().unwrap_or_default()
}

fn new_id() -> String {
    (JsDate::now() as u64).to_string()
}

// ── Storage write helper ─────────────────────────────────────────────────────

/// Write to localStorage, mapping failures to a user-facing message.
/// Writes can fail when the browser storage quota is full; surfacing it
/// matters because the in-memory state still looks saved while the data
/// would be lost on reload.
fn set_storage<T: Serialize + ?Sized>(key: &str, value: &T) -> Result<(), String> {
    LocalStorage::set(key, value).map_err(|e| {
        format!(
            "Salvataggio non riuscito ({}): {}. Esporta un backup e libera spazio sul dispositivo.",
            key, e
        )
    })
}

// ── User preferred scheda ────────────────────────────────────────────────────

pub fn load_user_preferred() -> Option<String> {
    LocalStorage::get::<String>("user_preferred_scheda").ok()
}

pub fn save_user_preferred(file: Option<&str>) -> Result<(), String> {
    match file {
        Some(f) => set_storage("user_preferred_scheda", f),
        None    => { LocalStorage::delete("user_preferred_scheda"); Ok(()) }
    }
}

// ── Schedule storage ─────────────────────────────────────────────────────────

pub fn load_schedules() -> Vec<Workout> {
    LocalStorage::get("schedules").unwrap_or_default()
}

fn save_schedules(schedules: &[Workout]) -> Result<(), String> {
    set_storage("schedules", schedules)
}

/// Replace an existing schedule (matched by id) with the fresh version, or insert if new.
/// Called every time a scheda is loaded — ensures localStorage always holds
/// the latest structure (new fields like `video`) without touching session data.
pub fn upsert_schedule(workout: &Workout) -> Result<(), String> {
    let mut schedules = load_schedules();
    let id = workout.id.clone();
    upsert_by(&mut schedules, workout.clone(), |s| s.id == id);
    save_schedules(&schedules)
}

// ── Sessions storage ─────────────────────────────────────────────────────────

fn sessions_key(workout_id: &str) -> String {
    format!("sessions__{}", workout_id)
}

pub fn load_sessions(workout_id: &str) -> Vec<Session> {
    LocalStorage::get(sessions_key(workout_id)).unwrap_or_default()
}

fn save_sessions(workout_id: &str, sessions: &[Session]) -> Result<(), String> {
    set_storage(&sessions_key(workout_id), sessions)
}

// ── Sessions index ───────────────────────────────────────────────────────────

pub fn load_sessions_index() -> Vec<SessionMeta> {
    LocalStorage::get("sessions_index").unwrap_or_default()
}

fn save_sessions_index(index: &[SessionMeta]) -> Result<(), String> {
    set_storage("sessions_index", index)
}

fn upsert_session_meta(meta: SessionMeta) -> Result<(), String> {
    let mut index = load_sessions_index();
    let id = meta.id.clone(); // capture before meta is moved
    upsert_by(&mut index, meta, |m| m.id == id);
    save_sessions_index(&index)
}

// ── Session helpers ───────────────────────────────────────────────────────────

/// Total number of sets expected for a day (sum of all `serie`).
pub fn total_day_sets(workout: &Workout, day_label: &str) -> u32 {
    workout.giorni.iter()
        .find(|d| d.giorno == day_label)
        .map(|d| d.esercizi.iter().map(|e| e.serie).sum())
        .unwrap_or(0)
}

/// Find the most recent non-terminated session for workout+day.
/// Returns None if no open session exists (use `create_session_for_day` to create one).
pub fn find_open_session(workout_id: &str, day_label: &str) -> Option<(String, Vec<CompletedSet>, usize)> {
    load_sessions(workout_id)
        .into_iter()
        .filter(|s| s.day == day_label && !s.done)
        .max_by(|a, b| a.updated.cmp(&b.updated))
        .map(|s| (s.id, s.sets, s.active_exercise))
}

/// All non-terminated session metas for a specific workout+day (for disambiguation).
pub fn open_sessions_for_day(workout_id: &str, day_label: &str) -> Vec<SessionMeta> {
    load_sessions_index()
        .into_iter()
        .filter(|m| m.workout_id == workout_id && m.day == day_label && !m.done)
        .collect()
}

/// How a day's open-session state resolves when (re)opening that day.
pub enum DaySession {
    /// No open session — start with a clean slate.
    Fresh,
    /// Exactly one open session — resume it.
    Resume {
        session_id: String,
        sets: Vec<CompletedSet>,
        active_exercise: usize,
    },
    /// Multiple open sessions — caller must show the resume/disambiguation dialog.
    Disambiguate(Vec<SessionMeta>),
}

/// Resolve the open-session state for `workout_id` + `day_label`:
/// 0 open sessions → `Fresh`, exactly 1 → `Resume`, more than 1 → `Disambiguate`.
/// Single source of truth for the branch that used to be copy-pasted across
/// auto-resume, day change, calendar select, and suggestion entry points.
pub fn resolve_day_session(workout_id: &str, day_label: &str) -> DaySession {
    let open = open_sessions_for_day(workout_id, day_label);
    match open.len() {
        0 => DaySession::Fresh,
        1 => match find_open_session(workout_id, day_label) {
            Some((session_id, sets, active_exercise)) => DaySession::Resume {
                session_id,
                sets,
                active_exercise,
            },
            None => DaySession::Fresh,
        },
        _ => DaySession::Disambiguate(open),
    }
}

/// Create and persist a brand-new session for a day. Called lazily on first set.
/// Idempotent: if an open session already exists for this day, returns its id.
/// On a failed write the session does NOT exist in storage — callers should
/// surface the error and avoid storing the id, so the next save retries.
pub fn create_session_for_day(workout: &Workout, day_idx: usize) -> Result<String, String> {
    let day = match workout.giorni.get(day_idx) {
        Some(d) => d,
        None => return Ok(new_id()),
    };
    // Safety net: don't create a second session if one already exists
    if let Some((existing_id, _, _)) = find_open_session(&workout.id, &day.giorno) {
        return Ok(existing_id);
    }
    let id  = new_id();
    let now = now_iso();
    let session = Session {
        id: id.clone(),
        workout_id: workout.id.clone(),
        workout_nome: workout.nome.clone(),
        day: day.giorno.clone(),
        started: now.clone(),
        updated: now.clone(),
        done: false,
        active_exercise: 0,
        sets: vec![],
    };
    // Write the full session first, the index after: a partial failure leaves
    // at worst an unindexed session, never a ghost index entry.
    let mut sessions = load_sessions(&workout.id);
    sessions.push(session);
    save_sessions(&workout.id, &sessions)?;
    upsert_session_meta(SessionMeta {
        id: id.clone(),
        workout_id: workout.id.clone(),
        workout_nome: workout.nome.clone(),
        day: day.giorno.clone(),
        started: now.clone(),
        updated: now,
        done: false,
        completion_pct: 0.0,
    })?;
    Ok(id)
}

/// Delete all non-terminated sessions for a specific workout+day.
pub fn delete_sessions_for_day(workout_id: &str, day_label: &str) -> Result<(), String> {
    let mut sessions = load_sessions(workout_id);
    let before = sessions.len();
    sessions.retain(|s| !(s.day == day_label && !s.done));
    if sessions.len() != before {
        save_sessions(workout_id, &sessions)?;
        let mut index = load_sessions_index();
        index.retain(|m| !(m.workout_id == workout_id && m.day == day_label && !m.done));
        save_sessions_index(&index)?;
    }
    Ok(())
}

/// Persist updated sets (and active_exercise) for an existing session,
/// and refresh the sessions_index entry.
pub fn update_session_sets(
    workout_id: &str,
    session_id: &str,
    sets: &[CompletedSet],
    active_exercise: usize,
    total_expected: u32,
) -> Result<(), String> {
    let mut sessions = load_sessions(workout_id);
    if let Some(s) = sessions.iter_mut().find(|s| s.id == session_id) {
        s.sets = sets.to_vec();
        s.active_exercise = active_exercise;
        s.updated = now_iso();
        let pct = s.completion_pct(total_expected);
        let meta = SessionMeta {
            id: session_id.to_string(),
            workout_id: workout_id.to_string(),
            workout_nome: s.workout_nome.clone(),
            day: s.day.clone(),
            started: s.started.clone(),
            updated: s.updated.clone(),
            done: s.done,
            completion_pct: pct,
        };
        save_sessions(workout_id, &sessions)?;
        upsert_session_meta(meta)?;
    }
    Ok(())
}

/// Mark a single session as terminated (done).
pub fn terminate_session(workout_id: &str, session_id: &str) -> Result<(), String> {
    let mut sessions = load_sessions(workout_id);
    let now = now_iso();
    if let Some(s) = sessions.iter_mut().find(|s| s.id == session_id) {
        s.done = true;
        s.updated = now.clone();
        save_sessions(workout_id, &sessions)?;
    }
    let mut index = load_sessions_index();
    if let Some(m) = index.iter_mut().find(|m| m.id == session_id) {
        m.done = true;
        m.updated = now.clone();
    }
    save_sessions_index(&index)
}

/// Remove a session entirely from storage.
pub fn delete_session(workout_id: &str, session_id: &str) -> Result<(), String> {
    let mut sessions = load_sessions(workout_id);
    sessions.retain(|s| s.id != session_id);
    save_sessions(workout_id, &sessions)?;
    let mut index = load_sessions_index();
    index.retain(|m| m.id != session_id);
    save_sessions_index(&index)
}

/// Terminated sessions older than this get pruned at startup (~24 months).
/// Generous on purpose: the weight chart keeps two years of history and the
/// export/backup remains the full archive.
const PRUNE_AFTER_DAYS: f64 = 730.0;

/// Delete terminated sessions whose last update is older than
/// [`PRUNE_AFTER_DAYS`], from both the per-workout stores and the index.
/// Bounds localStorage growth (writes fail silently-looking at quota).
/// Returns how many sessions were removed.
pub fn prune_old_done_sessions() -> Result<u32, String> {
    let cutoff_ms = JsDate::now() - PRUNE_AFTER_DAYS * 86_400_000.0;
    let cutoff = JsDate::new(&wasm_bindgen::JsValue::from_f64(cutoff_ms))
        .to_iso_string()
        .as_string()
        .unwrap_or_default();
    if cutoff.is_empty() { return Ok(0); }

    let mut index = load_sessions_index();
    // ISO-8601 UTC strings compare correctly as plain strings.
    let stale_ids: HashSet<String> = index.iter()
        .filter(|m| m.done && m.updated.as_str() < cutoff.as_str())
        .map(|m| m.id.clone())
        .collect();
    if stale_ids.is_empty() { return Ok(0); }

    let workout_ids: HashSet<String> = index.iter()
        .filter(|m| stale_ids.contains(&m.id))
        .map(|m| m.workout_id.clone())
        .collect();
    for wid in &workout_ids {
        let mut sessions = load_sessions(wid);
        sessions.retain(|s| !stale_ids.contains(&s.id));
        save_sessions(wid, &sessions)?;
    }
    index.retain(|m| !stale_ids.contains(&m.id));
    save_sessions_index(&index)?;
    Ok(stale_ids.len() as u32)
}

/// Insert or replace an item in a Vec. Uses `position` to avoid double-move.
fn upsert_by<T>(vec: &mut Vec<T>, item: T, matches: impl Fn(&T) -> bool) {
    match vec.iter().position(|x| matches(x)) {
        Some(i) => vec[i] = item,
        None    => vec.push(item),
    }
}

impl CatalogEntry {
    pub fn display_name(&self) -> String {
        let num = self.numero.clone().unwrap_or_default();
        if num.is_empty() { self.nome.clone() } else { format!("{} {}", self.nome, num) }
    }
    pub fn date_label(&self) -> String {
        format!("{} / {}",
            self.mese.clone().unwrap_or_default(),
            self.anno.clone().unwrap_or_default())
    }
}

/// Timer state passed as a single prop to ExerciseCard.
#[derive(Clone, PartialEq)]
pub struct TimerState {
    pub running: bool,
    pub left:    u32,
    pub total:   u32,
}

impl Session {
    pub fn completion_pct(&self, total_expected: u32) -> f32 {
        if total_expected == 0 { return 0.0; }
        (self.sets.len() as f32 / total_expected as f32 * 100.0).min(100.0)
    }
}

impl TimerState {
    #[allow(dead_code)]
    pub fn idle() -> Self { Self { running: false, left: 0, total: 0 } }
}

// ── Pure logic helpers ────────────────────────────────────────────────────────

/// Insert or update a CompletedSet in `list`. Sorts by set_number before returning.
pub fn upsert_completed_set(
    list: Vec<CompletedSet>,
    exercise: &Exercise,
    set_number: u32,
    peso: Option<f32>,
    reps: Option<String>,
    durata_min: Option<u32>,
) -> Vec<CompletedSet> {
    upsert_completed_set_at(list, exercise, set_number, peso, reps, durata_min, now_iso())
}

/// Pure core of [`upsert_completed_set`] with an injectable timestamp,
/// so it can be unit-tested natively (no JS `Date`).
pub fn upsert_completed_set_at(
    mut list: Vec<CompletedSet>,
    exercise: &Exercise,
    set_number: u32,
    peso: Option<f32>,
    reps: Option<String>,
    durata_min: Option<u32>,
    timestamp: String,
) -> Vec<CompletedSet> {
    if let Some(e) = list.iter_mut().find(|s| {
        s.exercise_id == exercise.id && s.set_number == set_number
    }) {
        e.peso       = peso;
        e.reps       = reps;
        e.durata_min = durata_min;
        e.timestamp  = timestamp;
    } else {
        list.push(CompletedSet {
            exercise_id: exercise.id.clone(),
            nome:        exercise.nome.clone(),
            set_number,
            peso,
            reps,
            durata_min,
            timestamp,
        });
    }
    list.sort_by_key(|s| s.set_number);
    list
}

/// Return the index of the next exercise in `day` that still has incomplete sets,
/// searching forward (wrapping) from `current_idx`.
/// Returns `current_idx` if all exercises are complete.
pub fn next_incomplete_exercise(
    day: &Day,
    sets: &[CompletedSet],
    current_idx: usize,
) -> usize {
    let n = day.esercizi.len();
    (1..n)
        .map(|off| (current_idx + off) % n)
        .find(|&i| {
            let ex = &day.esercizi[i];
            sets.iter().filter(|s| s.exercise_id == ex.id).count() < ex.serie as usize
        })
        .unwrap_or(current_idx)
}

/// Outcome of registering a set — describes what the caller must apply to its
/// UI state handles. Returned by [`register_set`] so the persistence logic lives
/// in one place while the view layer only mirrors the result into `use_state`s.
pub struct SetRegistration {
    /// Updated sets list to store in `saved_sets`.
    pub sets: Vec<CompletedSet>,
    /// Index of the exercise that should now be active.
    pub next_active_exercise: usize,
    /// Session id used (created lazily if none existed).
    pub session_id: String,
    /// `true` when a new session was created — caller must update `current_session_id`.
    pub session_created: bool,
    /// When `Some((idx, value))`, caller should prefill `weight_inputs` at `idx`.
    pub prefill_weight: Option<(usize, String)>,
    /// When `Some(msg)`, persisting to localStorage failed (quota?) — the
    /// returned sets are still good for the UI, but the caller must surface
    /// the error: the data will NOT survive a reload.
    pub storage_error: Option<String>,
}

/// Register `set_number` (1-based) of `exercise` with the given peso/reps and
/// persist it: upserts the set, advances the active exercise when the current one
/// is complete, lazily creates the session, updates storage, and computes whether
/// the next set's weight input should be pre-filled.
///
/// This is the single source of truth shared by manual save, skip-timer, and the
/// recovery-timer auto-save. Callers own only the *policy* differences (which set
/// number, whether the weight uses fallback, timer teardown) and then mirror the
/// returned [`SetRegistration`] into their state handles.
#[allow(clippy::too_many_arguments)]
pub fn register_set(
    workout: &Workout,
    day: &Day,
    day_index: usize,
    exercise: &Exercise,
    current_exercise_idx: usize,
    set_number: u32,
    peso: Option<f32>,
    reps: Option<String>,
    weight_str: &str,
    prior_sets: Vec<CompletedSet>,
    weight_inputs: &HashMap<String, Vec<String>>,
    current_session_id: &str,
) -> SetRegistration {
    let list = upsert_completed_set(prior_sets, exercise, set_number, peso, reps, None);

    let ex_done = list.iter()
        .filter(|s| s.exercise_id == exercise.id)
        .count() >= exercise.serie as usize;
    let next_active_exercise = if ex_done {
        next_incomplete_exercise(day, &list, current_exercise_idx)
    } else {
        current_exercise_idx
    };

    // On a failed session creation keep session_id empty so the caller doesn't
    // store a bogus id and the next save retries the creation.
    let (session_id, session_created, mut storage_error) = if current_session_id.is_empty() {
        match create_session_for_day(workout, day_index) {
            Ok(id) => (id, true, None),
            Err(e) => (String::new(), false, Some(e)),
        }
    } else {
        (current_session_id.to_string(), false, None)
    };

    if !session_id.is_empty() {
        let total = total_day_sets(workout, &day.giorno);
        if let Err(e) =
            update_session_sets(&workout.id, &session_id, &list, next_active_exercise, total)
        {
            storage_error.get_or_insert(e);
        }
    }

    let prefill_weight =
        compute_prefill_weight(weight_inputs, &exercise.id, exercise.serie, set_number, weight_str);

    SetRegistration {
        sets: list,
        next_active_exercise,
        session_id,
        session_created,
        prefill_weight,
        storage_error,
    }
}

/// Decide whether the weight just used for `set_number` (1-based) should
/// pre-fill the NEXT set's input slot: only when a next set exists, the weight
/// is non-empty and the next slot hasn't already been typed ahead by the user.
pub fn compute_prefill_weight(
    weight_inputs: &HashMap<String, Vec<String>>,
    exercise_id: &str,
    serie: u32,
    set_number: u32,
    weight_str: &str,
) -> Option<(usize, String)> {
    let next_idx = set_number as usize; // 1-based set number == 0-based index of the next set
    let next_slot_filled = weight_inputs.get(exercise_id)
        .and_then(|v| v.get(next_idx))
        .map(|v| !v.is_empty())
        .unwrap_or(false);
    if next_idx < serie as usize && !weight_str.is_empty() && !next_slot_filled {
        Some((next_idx, weight_str.to_string()))
    } else {
        None
    }
}

/// Read the input value for `exercise_id` at `idx`, falling back to the most
/// recent non-empty value at a lower index, then to `default`.
pub fn get_input_with_fallback(
    map: &HashMap<String, Vec<String>>,
    exercise_id: &str,
    idx: usize,
    default: &str,
) -> String {
    map.get(exercise_id)
        .and_then(|v| v.get(idx).cloned())
        .filter(|v| !v.is_empty())
        .or_else(|| {
            (0..idx).rev().find_map(|prev| {
                map.get(exercise_id)
                    .and_then(|v| v.get(prev).cloned())
                    .filter(|v| !v.is_empty())
            })
        })
        .unwrap_or_else(|| default.to_string())
}

/// Resize the inner Vec for `exercise_id` if needed, then set the value at `idx`.
pub fn update_input_map(
    mut map: HashMap<String, Vec<String>>,
    exercise_id: String,
    idx: usize,
    value: String,
) -> HashMap<String, Vec<String>> {
    let entry = map.entry(exercise_id).or_default();
    if entry.len() <= idx { entry.resize(idx + 1, String::new()); }
    entry[idx] = value;
    map
}

// ── Weight history ────────────────────────────────────────────────────────────

/// One data point for the weight-progression chart.
#[derive(Clone)]
pub struct WeightPoint {
    pub date:       String,  // "YYYY-MM-DD"
    pub max_weight: f32,
}

/// Collect the max weight used per terminated session for an exercise.
/// Searches all sessions for the workout, regardless of day.
pub fn weight_history_for_exercise(workout_id: &str, exercise_id: &str) -> Vec<WeightPoint> {
    let mut points: Vec<WeightPoint> = load_sessions(workout_id)
        .into_iter()
        .filter(|s| s.done)
        .filter_map(|s| {
            let max_w = s.sets.iter()
                .filter(|set| set.exercise_id == exercise_id)
                .filter_map(|set| set.peso)
                .filter(|w| *w > 0.0)
                .fold(f32::NEG_INFINITY, f32::max);
            if max_w == f32::NEG_INFINITY { return None; }
            let date = s.started.get(..10).unwrap_or(&s.started).to_string();
            Some(WeightPoint { date, max_weight: max_w })
        })
        .collect();
    points.sort_by(|a, b| a.date.cmp(&b.date));
    points
}

/// All terminated sessions for a workout+day, newest first.
pub fn terminated_sessions_for_day(workout_id: &str, day_label: &str) -> Vec<Session> {
    let mut sessions: Vec<Session> = load_sessions(workout_id)
        .into_iter()
        .filter(|s| s.day == day_label && s.done)
        .collect();
    sessions.sort_by(|a, b| b.updated.cmp(&a.updated));
    sessions
}

// ── Export / Import ───────────────────────────────────────────────────────────

#[derive(Serialize, Deserialize, Debug)]
pub struct ExportData {
    pub version: u32,
    pub exported_at: String,
    pub schedules: Vec<Workout>,
    pub sessions_index: Vec<SessionMeta>,
    pub sessions: std::collections::HashMap<String, Vec<Session>>,
}

/// Serialize all localStorage data into a pretty-printed JSON string.
pub fn export_all_data() -> String {
    let schedules      = load_schedules();
    let sessions_index = load_sessions_index();
    // Collect sessions for every known workout_id
    let ids: std::collections::HashSet<String> = sessions_index
        .iter().map(|m| m.workout_id.clone()).collect();
    let mut sessions = std::collections::HashMap::new();
    for id in ids {
        let s = load_sessions(&id);
        if !s.is_empty() { sessions.insert(id, s); }
    }
    let data = ExportData {
        version: 1,
        exported_at: now_iso(),
        schedules,
        sessions_index,
        sessions,
    };
    serde_json::to_string_pretty(&data).unwrap_or_default()
}

/// Parse an export file and overwrite all localStorage data.
pub fn import_all_data(json: &str) -> Result<(), String> {
    let data: ExportData = serde_json::from_str(json)
        .map_err(|e| format!("Formato non riconosciuto: {}", e))?;
    save_schedules(&data.schedules)?;
    save_sessions_index(&data.sessions_index)?;
    for (workout_id, s) in &data.sessions {
        save_sessions(workout_id, s)?;
    }
    Ok(())
}

// ── Calendar / suggestion ────────────────────────────────────────────────────

/// Display info shown on the calendar's "next workout" CTA button.
#[derive(Clone, PartialEq)]
pub struct SuggestionInfo {
    pub workout_nome: String,
    pub day_label: String,
}

pub fn find_session_by_id(workout_id: &str, session_id: &str) -> Option<Session> {
    load_sessions(workout_id).into_iter().find(|s| s.id == session_id)
}

/// Returns (Workout, day_index) for the next suggested training based on the
/// preferred scheda. Bridges catalog entry → workout via `nome` field matching.
/// Falls back to day 0 when: no sessions for this scheda, or last session > 30 days ago.
pub fn compute_suggestion_workout(
    sessions: &[SessionMeta],
    schedules: &[Workout],
    catalog: &[CatalogEntry],
    user_preferred: &Option<String>,
) -> Option<(Workout, usize)> {
    let pref = match user_preferred {
        Some(file) => catalog.iter().find(|e| &e.file == file),
        None       => catalog.iter().find(|e| e.preferita.unwrap_or(false)),
    }?;

    // Fuzzy match: exact nome, or one is a prefix of the other.
    // Handles cases where catalog.json uses a shorter display name than the
    // full nome in the workout JSON (e.g. "Scheda X" vs "Scheda X (v2)").
    let workout = schedules.iter().find(|w| {
        w.nome == pref.nome
            || w.nome.starts_with(pref.nome.as_str())
            || pref.nome.starts_with(w.nome.as_str())
    })?.clone();

    let mut done: Vec<&SessionMeta> = sessions.iter()
        .filter(|s| s.done && s.workout_id == workout.id)
        .collect();
    done.sort_by(|a, b| a.started.cmp(&b.started));

    let day_idx = if let Some(last) = done.last() {
        let days_ago = (JsDate::now() - JsDate::parse(&last.started)) / 86_400_000.0;
        if days_ago > 30.0 {
            0
        } else {
            let cur = workout.giorni.iter().position(|d| d.giorno == last.day).unwrap_or(0);
            (cur + 1) % workout.giorni.len().max(1)
        }
    } else {
        0
    };

    Some((workout, day_idx))
}

pub fn compute_suggestion(
    sessions: &[SessionMeta],
    schedules: &[Workout],
    catalog: &[CatalogEntry],
    user_preferred: &Option<String>,
) -> Option<SuggestionInfo> {
    let (workout, day_idx) = compute_suggestion_workout(sessions, schedules, catalog, user_preferred)?;
    let day = workout.giorni.get(day_idx)?;
    Some(SuggestionInfo {
        workout_nome: workout.nome.clone(),
        day_label: day.etichetta.clone().unwrap_or_else(|| day.giorno.clone()),
    })
}

// ── Reps helpers ─────────────────────────────────────────────────────────────

/// Parse a reps target string like "8-10" or "12" into (min, max).
pub fn parse_reps_range(reps: &str) -> (i32, i32) {
    let clean = reps.trim();
    if let Some((a, b)) = clean.split_once('-') {
        let lo = a.trim().parse().unwrap_or(0);
        let hi = b.trim().parse().unwrap_or(lo);
        (lo, hi)
    } else {
        let n = clean.parse().unwrap_or(0);
        (n, n)
    }
}

// ── Unit tests (native: `cargo test` from app/) ──────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn ex(id: &str, serie: u32) -> Exercise {
        Exercise {
            id: id.into(),
            nome: id.to_uppercase(),
            serie,
            reps: "8-10".into(),
            recupero: Some(90),
            note: None,
            video: None,
            tipo: None,
            durata: None,
        }
    }

    fn done_set(eid: &str, n: u32) -> CompletedSet {
        CompletedSet {
            exercise_id: eid.into(),
            nome: eid.to_uppercase(),
            set_number: n,
            peso: Some(10.0),
            reps: Some("8".into()),
            timestamp: "t".into(),
            durata_min: None,
        }
    }

    fn day(esercizi: Vec<Exercise>) -> Day {
        Day { giorno: "A".into(), etichetta: None, esercizi }
    }

    fn inputs(eid: &str, vals: &[&str]) -> HashMap<String, Vec<String>> {
        let mut m = HashMap::new();
        m.insert(eid.to_string(), vals.iter().map(|s| s.to_string()).collect());
        m
    }

    // ── parse_reps_range ─────────────────────────────────────────────────────

    #[test]
    fn reps_range_plain_number() {
        assert_eq!(parse_reps_range("12"), (12, 12));
    }

    #[test]
    fn reps_range_dash() {
        assert_eq!(parse_reps_range("8-10"), (8, 10));
    }

    #[test]
    fn reps_range_spaces() {
        assert_eq!(parse_reps_range(" 8 - 10 "), (8, 10));
    }

    #[test]
    fn reps_range_open_ended_uses_lo() {
        assert_eq!(parse_reps_range("8-"), (8, 8));
    }

    #[test]
    fn reps_range_non_numeric_is_zero() {
        assert_eq!(parse_reps_range("max"), (0, 0));
    }

    // ── get_input_with_fallback ──────────────────────────────────────────────

    #[test]
    fn input_direct_hit() {
        let m = inputs("e1", &["20", "22.5"]);
        assert_eq!(get_input_with_fallback(&m, "e1", 1, ""), "22.5");
    }

    #[test]
    fn input_falls_back_to_most_recent_lower() {
        let m = inputs("e1", &["20", "", ""]);
        assert_eq!(get_input_with_fallback(&m, "e1", 2, ""), "20");
    }

    #[test]
    fn input_fallback_prefers_nearest() {
        let m = inputs("e1", &["20", "25", ""]);
        assert_eq!(get_input_with_fallback(&m, "e1", 2, ""), "25");
    }

    #[test]
    fn input_default_when_all_empty() {
        let m = inputs("e1", &["", ""]);
        assert_eq!(get_input_with_fallback(&m, "e1", 1, "8-10"), "8-10");
    }

    #[test]
    fn input_default_when_key_missing() {
        let m = HashMap::new();
        assert_eq!(get_input_with_fallback(&m, "e1", 0, "x"), "x");
    }

    // ── update_input_map ─────────────────────────────────────────────────────

    #[test]
    fn update_map_resizes_and_sets() {
        let m = update_input_map(HashMap::new(), "e1".into(), 2, "30".into());
        assert_eq!(m["e1"], vec!["", "", "30"]);
    }

    #[test]
    fn update_map_overwrites_in_place() {
        let m = inputs("e1", &["20", "25"]);
        let m = update_input_map(m, "e1".into(), 0, "21".into());
        assert_eq!(m["e1"], vec!["21", "25"]);
    }

    // ── next_incomplete_exercise ─────────────────────────────────────────────

    #[test]
    fn next_exercise_skips_complete_and_wraps() {
        // e0 complete, e1 complete, e2 incomplete; from e1 → e2
        let d = day(vec![ex("e0", 1), ex("e1", 1), ex("e2", 2)]);
        let sets = vec![done_set("e0", 1), done_set("e1", 1), done_set("e2", 1)];
        assert_eq!(next_incomplete_exercise(&d, &sets, 1), 2);
        // from e2 (still incomplete itself): search starts FORWARD, wraps past
        // complete e0/e1 and returns current_idx because nothing else is open
        assert_eq!(next_incomplete_exercise(&d, &sets, 2), 2);
    }

    #[test]
    fn next_exercise_wraps_to_earlier() {
        // e0 incomplete, e1 complete; from e1 → wraps to e0
        let d = day(vec![ex("e0", 2), ex("e1", 1)]);
        let sets = vec![done_set("e0", 1), done_set("e1", 1)];
        assert_eq!(next_incomplete_exercise(&d, &sets, 1), 0);
    }

    #[test]
    fn next_exercise_all_done_returns_current() {
        let d = day(vec![ex("e0", 1), ex("e1", 1)]);
        let sets = vec![done_set("e0", 1), done_set("e1", 1)];
        assert_eq!(next_incomplete_exercise(&d, &sets, 0), 0);
    }

    // ── upsert_completed_set_at ──────────────────────────────────────────────

    #[test]
    fn upsert_inserts_sorted() {
        let e = ex("e1", 3);
        let list = upsert_completed_set_at(vec![], &e, 3, Some(50.0), None, None, "t1".into());
        let list = upsert_completed_set_at(list, &e, 1, Some(40.0), None, None, "t2".into());
        let nums: Vec<u32> = list.iter().map(|s| s.set_number).collect();
        assert_eq!(nums, vec![1, 3]);
    }

    #[test]
    fn upsert_updates_existing_in_place() {
        let e = ex("e1", 3);
        let list = upsert_completed_set_at(vec![], &e, 1, Some(40.0), Some("8".into()), None, "t1".into());
        let list = upsert_completed_set_at(list, &e, 1, Some(45.0), Some("6".into()), None, "t2".into());
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].peso, Some(45.0));
        assert_eq!(list[0].reps.as_deref(), Some("6"));
        assert_eq!(list[0].timestamp, "t2");
    }

    #[test]
    fn upsert_distinguishes_exercises() {
        let e1 = ex("e1", 3);
        let e2 = ex("e2", 3);
        let list = upsert_completed_set_at(vec![], &e1, 1, Some(40.0), None, None, "t".into());
        let list = upsert_completed_set_at(list, &e2, 1, Some(60.0), None, None, "t".into());
        assert_eq!(list.len(), 2);
    }

    // ── compute_prefill_weight ───────────────────────────────────────────────

    #[test]
    fn prefill_propagates_to_empty_next_slot() {
        let m = inputs("e1", &["40"]);
        assert_eq!(
            compute_prefill_weight(&m, "e1", 3, 1, "40"),
            Some((1, "40".to_string()))
        );
    }

    #[test]
    fn prefill_respects_typed_ahead_value() {
        let m = inputs("e1", &["40", "45"]);
        assert_eq!(compute_prefill_weight(&m, "e1", 3, 1, "40"), None);
    }

    #[test]
    fn prefill_skips_last_set() {
        let m = inputs("e1", &["40", "40", "40"]);
        assert_eq!(compute_prefill_weight(&m, "e1", 3, 3, "40"), None);
    }

    #[test]
    fn prefill_skips_empty_weight() {
        let m = inputs("e1", &[""]);
        assert_eq!(compute_prefill_weight(&m, "e1", 3, 1, ""), None);
    }

    #[test]
    fn prefill_works_with_missing_slot_vec() {
        // No input vec at all for the exercise: next slot counts as empty.
        let m = HashMap::new();
        assert_eq!(
            compute_prefill_weight(&m, "e1", 3, 1, "40"),
            Some((1, "40".to_string()))
        );
    }

    // ── completion_pct / total_day_sets ──────────────────────────────────────

    #[test]
    fn completion_pct_clamps_and_handles_zero() {
        let mut s = Session {
            id: "1".into(), workout_id: "w".into(), workout_nome: "W".into(),
            day: "A".into(), started: "t".into(), updated: "t".into(),
            done: false, active_exercise: 0,
            sets: vec![done_set("e1", 1), done_set("e1", 2)],
        };
        assert_eq!(s.completion_pct(0), 0.0);
        assert_eq!(s.completion_pct(4), 50.0);
        s.sets.push(done_set("e1", 3));
        s.sets.push(done_set("e1", 4));
        s.sets.push(done_set("e1", 5)); // over-complete
        assert_eq!(s.completion_pct(4), 100.0);
    }

    #[test]
    fn total_day_sets_sums_serie() {
        let w = Workout {
            id: "w".into(), nome: "W".into(), descrizione: None, categoria: None,
            giorni: vec![day(vec![ex("e0", 3), ex("e1", 4)])],
        };
        assert_eq!(total_day_sets(&w, "A"), 7);
        assert_eq!(total_day_sets(&w, "B"), 0);
    }
}

