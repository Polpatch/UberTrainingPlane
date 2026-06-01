use gloo_file::callbacks::{read_as_text, FileReader};
use gloo_file::File as GlooFile;
use gloo_storage::{LocalStorage, Storage};
use gloo_net::http::Request;
use gloo_timers::callback::Interval;
use js_sys::Date;
use serde::{Deserialize, Serialize};
use std::cell::Cell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;
use web_sys::HtmlInputElement;
use yew::prelude::*;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
struct Workout {
    id: String,
    nome: String,
    descrizione: Option<String>,
    categoria: Option<String>,
    giorni: Vec<Day>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
struct Day {
    giorno: String,
    etichetta: Option<String>,
    esercizi: Vec<Exercise>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
struct Exercise {
    id: String,
    nome: String,
    serie: u32,
    reps: String,
    recupero: Option<u32>,
    note: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
struct CatalogEntry {
    file: String,
    nome: String,
    numero: Option<String>,
    mese: Option<String>,
    anno: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
struct CompletedSet {
    exercise_id: String,
    nome: String,
    set_number: u32,
    peso: Option<f32>,
    reps: Option<String>,
    timestamp: String,
}

fn session_key(workout_id: &str, day_label: &str) -> String {
    format!("workout_session__{}__{}", workout_id, day_label.replace(' ', "_"))
}

fn load_session(key: &str) -> Vec<CompletedSet> {
    LocalStorage::get(key).unwrap_or_else(|_| Vec::new())
}

#[function_component(App)]
fn app() -> Html {
    let workout = use_state(|| None::<Workout>);
    let error = use_state(|| None::<String>);
    let day_index = use_state(|| 0usize);
    let selected_exercise = use_state(|| 0usize);
    let weight_inputs = use_state(|| HashMap::<String, Vec<String>>::new());
    let reps_inputs = use_state(|| HashMap::<String, Vec<String>>::new());
    let saved_sets = use_state(|| Vec::<CompletedSet>::new());
    let catalog = use_state(|| Vec::<CatalogEntry>::new());
    let catalog_loading = use_state(|| true);
    let timer_running = use_state(|| false);
    let timer_left = use_state(|| 0u32);
    let timer_total = use_state(|| 0u32);
    let timer_handle = use_mut_ref(|| None::<Interval>);
    let reader_task = use_mut_ref(|| None::<FileReader>);

    let _fetch_catalog = {
        let catalog = catalog.clone();
        let error = error.clone();
        let catalog_loading = catalog_loading.clone();

        use_effect_with_deps(
            move |_| {
                spawn_local(async move {
                    match Request::get("schede/catalog.json").send().await {
                        Ok(resp) => {
                            if resp.ok() {
                                match resp.json::<Vec<CatalogEntry>>().await {
                                    Ok(list) => {
                                        catalog.set(list);
                                    }
                                    Err(err) => {
                                        error.set(Some(format!("Errore catalogo: {:?}", err)));
                                    }
                                }
                            } else {
                                error.set(Some(format!("Errore caricamento catalogo: {}", resp.status())));
                            }
                            catalog_loading.set(false);
                        }
                        Err(err) => {
                            error.set(Some(format!("Errore caricamento catalogo: {:?}", err)));
                            catalog_loading.set(false);
                        }
                    }
                });
                || ()
            },
            (),
        )
    };

    let on_file_change = {
        let workout = workout.clone();
        let error = error.clone();
        let selected_exercise = selected_exercise.clone();
        let saved_sets = saved_sets.clone();
        let reader_task = reader_task.clone();

        Callback::from(move |event: Event| {
            let input = event
                .target()
                .and_then(|t| t.dyn_into::<HtmlInputElement>().ok());

            if let Some(input) = input {
                if let Some(file) = input.files().and_then(|files| files.get(0)) {
                    let gloo_file = GlooFile::from(file);
                    let workout = workout.clone();
                    let error = error.clone();
                    let selected_exercise = selected_exercise.clone();
                    let saved_sets = saved_sets.clone();

                    let task = read_as_text(&gloo_file, move |result| {
                        match result {
                            Ok(text) => match serde_json::from_str::<Workout>(&text) {
                                Ok(data) => {
                                    let saved = data
                                        .giorni
                                        .get(0)
                                        .map(|day| load_session(&session_key(&data.id, &day.giorno)))
                                        .unwrap_or_else(Vec::new);
                                    workout.set(Some(data));
                                    error.set(None);
                                    selected_exercise.set(0);
                                    saved_sets.set(saved);
                                }
                                Err(err) => {
                                    error.set(Some(format!("Errore JSON: {}", err)));
                                }
                            },
                            Err(err) => {
                                error.set(Some(format!("Errore lettura file: {:?}", err)));
                            }
                        }
                    });

                    *reader_task.borrow_mut() = Some(task);
                }
            }
        })
    };

    let on_select_exercise = {
        let selected_exercise = selected_exercise.clone();
        Callback::from(move |idx: usize| selected_exercise.set(idx))
    };

    let on_load_catalog_entry = {
        let workout = workout.clone();
        let error = error.clone();
        let selected_exercise = selected_exercise.clone();
        let saved_sets = saved_sets.clone();
        Callback::from(move |entry: CatalogEntry| {
            let workout = workout.clone();
            let error = error.clone();
            let selected_exercise = selected_exercise.clone();
            let saved_sets = saved_sets.clone();
            let file_path = entry.file.clone();
            spawn_local(async move {
                match Request::get(&file_path).send().await {
                    Ok(resp) => {
                        if resp.ok() {
                            match resp.text().await {
                                Ok(text) => match serde_json::from_str::<Workout>(&text) {
                                    Ok(data) => {
                                        let saved = data
                                            .giorni
                                            .get(0)
                                            .map(|day| load_session(&session_key(&data.id, &day.giorno)))
                                            .unwrap_or_else(Vec::new);
                                        workout.set(Some(data));
                                        error.set(None);
                                        selected_exercise.set(0);
                                        saved_sets.set(saved);
                                    }
                                    Err(err) => {
                                        error.set(Some(format!("Errore JSON: {}", err)));
                                    }
                                },
                                Err(err) => {
                                    error.set(Some(format!("Errore caricamento file: {:?}", err)));
                                }
                            }
                        } else {
                            error.set(Some(format!("Errore caricamento file: {}", resp.status())));
                        }
                    }
                    Err(err) => {
                        error.set(Some(format!("Errore caricamento file: {:?}", err)));
                    }
                }
            });
        })
    };

    let on_change_day = {
        let workout = workout.clone();
        let day_index = day_index.clone();
        let saved_sets = saved_sets.clone();
        Callback::from(move |idx: usize| {
            day_index.set(idx);
            if let Some(workout) = &*workout {
                if let Some(day) = workout.giorni.get(idx) {
                    saved_sets.set(load_session(&session_key(&workout.id, &day.giorno)));
                }
            }
        })
    };

    let on_weight_change = {
        let weight_inputs = weight_inputs.clone();
        Callback::from(move |(exercise_id, idx, value): (String, usize, String)| {
            let mut map = (*weight_inputs).clone();
            let entry = map.entry(exercise_id.clone()).or_insert_with(Vec::new);
            if entry.len() <= idx {
                entry.resize(idx + 1, String::new());
            }
            entry[idx] = value;
            weight_inputs.set(map);
        })
    };

    let on_reps_change = {
        let reps_inputs = reps_inputs.clone();
        Callback::from(move |(exercise_id, idx, value): (String, usize, String)| {
            let mut map = (*reps_inputs).clone();
            let entry = map.entry(exercise_id.clone()).or_insert_with(Vec::new);
            if entry.len() <= idx {
                entry.resize(idx + 1, String::new());
            }
            entry[idx] = value;
            reps_inputs.set(map);
        })
    };

    let on_save_set = {
        let workout = workout.clone();
        let day_index = day_index.clone();
        let selected_exercise = selected_exercise.clone();
        let weight_inputs = weight_inputs.clone();
        let reps_inputs = reps_inputs.clone();
        let saved_sets = saved_sets.clone();

        Callback::from(move |set_index: usize| {
            if let Some(workout) = &*workout {
                if let Some(day) = workout.giorni.get(*day_index) {
                    if let Some(exercise) = day.esercizi.get(*selected_exercise) {
                        let weight = weight_inputs
                            .get(&exercise.id)
                            .and_then(|values| values.get(set_index))
                            .and_then(|value| value.parse::<f32>().ok());
                        let reps = reps_inputs
                            .get(&exercise.id)
                            .and_then(|values| values.get(set_index))
                            .cloned();
                        let set_number = (set_index + 1) as u32;
                        let timestamp = Date::new_0()
                            .to_iso_string()
                            .as_string()
                            .unwrap_or_else(|| "".into());

                        let mut list = (*saved_sets).clone();
                        if let Some(existing) = list.iter_mut().find(|item| item.exercise_id == exercise.id && item.set_number == set_number) {
                            existing.peso = weight;
                            existing.reps = reps.clone();
                            existing.timestamp = timestamp;
                        } else {
                            list.push(CompletedSet {
                                exercise_id: exercise.id.clone(),
                                nome: exercise.nome.clone(),
                                set_number,
                                peso: weight,
                                reps,
                                timestamp,
                            });
                        }
                        list.sort_by(|a, b| a.set_number.cmp(&b.set_number));
                        let key = session_key(&workout.id, &day.giorno);
                        let _ = LocalStorage::set(&key, &list);
                        saved_sets.set(list);
                    }
                }
            }
        })
    };

    let on_start_timer = {
        let workout_state = workout.clone();
        let day_index = day_index.clone();
        let selected_exercise = selected_exercise.clone();
        let timer_running = timer_running.clone();
        let timer_left = timer_left.clone();
        let timer_total = timer_total.clone();
        let timer_handle = timer_handle.clone();
        let saved_sets = saved_sets.clone();
        let weight_inputs = weight_inputs.clone();
        let reps_inputs = reps_inputs.clone();

        Callback::from(move |_| {
            if *timer_running {
                return;
            }
            if let Some(workout) = &*workout_state {
                if let Some(day) = workout.giorni.get(*day_index) {
                    if let Some(exercise) = day.esercizi.get(*selected_exercise) {
                        let duration = exercise.recupero.unwrap_or(90);
                        timer_left.set(duration);
                        timer_total.set(duration);

                        timer_handle.borrow_mut().take();
                        let timer_left_state = timer_left.clone();
                        let timer_running_inner = timer_running.clone();
                        let timer_handle_inner = timer_handle.clone();

                        // clone state handles for the interval closure so outer callback stays Fn
                        let workout_for_timer = workout_state.clone();
                        let day_index_for_timer = day_index.clone();
                        let selected_exercise_for_timer = selected_exercise.clone();
                        let saved_sets_for_timer = saved_sets.clone();
                        let weight_inputs_for_timer = weight_inputs.clone();
                        let reps_inputs_for_timer = reps_inputs.clone();

                        let remaining_counter = Rc::new(Cell::new(duration));
                        let remaining_counter_clone = remaining_counter.clone();

                        let handle = Interval::new(1000, move || {
                            let next = remaining_counter_clone.get().saturating_sub(1);
                            remaining_counter_clone.set(next);
                            timer_left_state.set(next);
                            web_sys::console::log_1(&format!("timer tick: next={}", next).into());

                            if next == 0 {
                                timer_running_inner.set(false);
                                if let Some(workout) = &*workout_for_timer {
                                    if let Some(day) = workout.giorni.get(*day_index_for_timer) {
                                                            if let Some(exercise) = day.esercizi.get(*selected_exercise_for_timer) {
                                                let existing_numbers: HashSet<u32> = (*saved_sets_for_timer)
                                                    .iter()
                                                    .filter(|item| item.exercise_id == exercise.id)
                                                    .map(|item| item.set_number)
                                                    .collect();
                                                let next_set = (1..=exercise.serie)
                                                    .find(|n| !existing_numbers.contains(n))
                                                    .unwrap_or(existing_numbers.len() as u32 + 1);
                                                let next_index = (next_set - 1) as usize;
                                                let weight = weight_inputs_for_timer
                                                    .get(&exercise.id)
                                                    .and_then(|values| values.get(next_index))
                                                    .and_then(|value| value.parse::<f32>().ok());
                                                let reps = reps_inputs_for_timer
                                                    .get(&exercise.id)
                                                    .and_then(|values| values.get(next_index))
                                                    .cloned();
                                                let entry = CompletedSet {
                                                    exercise_id: exercise.id.clone(),
                                                    nome: exercise.nome.clone(),
                                                    set_number: next_set,
                                                    peso: weight,
                                                    reps,
                                                    timestamp: Date::new_0().to_iso_string().as_string().unwrap_or_else(|| "".into()),
                                                };
                                                let mut list = (*saved_sets_for_timer).clone();
                                                if let Some(existing) = list.iter_mut().find(|item| item.exercise_id == exercise.id && item.set_number == next_set) {
                                                    *existing = entry;
                                                } else {
                                                    list.push(entry);
                                                }
                                                list.sort_by(|a, b| a.set_number.cmp(&b.set_number));
                                                let key = session_key(&workout.id, &day.giorno);
                                                let _ = LocalStorage::set(&key, &list);
                                                saved_sets_for_timer.set(list);
                                                web_sys::console::log_1(&"timer: saved set".into());
                                            }
                                    }
                                }
                                timer_handle_inner.borrow_mut().take();
                            }
                        });
                        *timer_handle.borrow_mut() = Some(handle);
                        timer_running.set(true);
                    }
                }
            }
        })
    };

    let clear_workout = {
        let workout = workout.clone();
        let error = error.clone();
        let day_index = day_index.clone();
        let selected_exercise = selected_exercise.clone();
        let saved_sets = saved_sets.clone();
        let weight_inputs = weight_inputs.clone();
        let reps_inputs = reps_inputs.clone();
        let timer_running = timer_running.clone();
        let timer_handle = timer_handle.clone();
        Callback::from(move |_| {
            if timer_handle.borrow_mut().is_some() {
                timer_handle.borrow_mut().take();
            }
            timer_running.set(false);
            workout.set(None);
            error.set(None);
            day_index.set(0);
            selected_exercise.set(0);
            saved_sets.set(Vec::new());
            weight_inputs.set(HashMap::new());
            reps_inputs.set(HashMap::new());
        })
    };

    let current_session = saved_sets.clone();

    html! {
        <div class="app-shell">
            <header class="app-header">
                <div>
                    <h1>{"Allenamento WASM"}</h1>
                    <p>{"Carica una scheda JSON e segui l'allenamento in tempo reale."}</p>
                </div>
                if workout.is_some() {
                    <button class="clear-button" onclick={clear_workout}>{"Carica un'altra scheda"}</button>
                }
            </header>

            <main class="app-main">
                {
                    if let Some(workout_data) = &*workout {
                        let day = workout_data.giorni.get(*day_index);
                        html! {
                            <div class="workout-details">
                                <section class="workout-meta">
                                    <div class="meta-label">{format!("Scheda: {}", workout_data.nome)}</div>
                                    if let Some(desc) = &workout_data.descrizione {
                                        <p class="meta-desc">{desc.clone()}</p>
                                    }
                                    if let Some(cat) = &workout_data.categoria {
                                        <div class="meta-tag">{cat.clone()}</div>
                                    }
                                </section>

                                <section class="day-tabs">
                                    { for workout_data.giorni.iter().enumerate().map(|(idx, day)| {
                                        let selected = *day_index == idx;
                                        let on_change_day = on_change_day.clone();
                                        let onclick = Callback::from(move |_| on_change_day.emit(idx));
                                        html! {
                                            <button class={classes!("day-tab", if selected { Some("active") } else { None })} {onclick}>
                                                {&day.giorno}
                                            </button>
                                        }
                                    }) }
                                </section>

                                {
                                    if let Some(day) = day {
                                        let selected = *selected_exercise;
                                        html! {
                                            <>
                                                <div class="day-header">
                                                    <h2>{ day.etichetta.clone().unwrap_or_else(|| day.giorno.clone()) }</h2>
                                                    <p>{ format!("{} esercizi", day.esercizi.len()) }</p>
                                                </div>

                                                <section class="exercise-list">
                                                    { for day.esercizi.iter().enumerate().map(|(idx, exercise)| {
                                                        let selected = selected == idx;
                                                        let onclick = {
                                                            let on_select_exercise = on_select_exercise.clone();
                                                            Callback::from(move |_| on_select_exercise.emit(idx))
                                                        };
                                                        let exercise_id = exercise.id.clone();
                                                        let on_save_set = on_save_set.clone();
                                                        html! {
                                                            <article class={classes!("exercise-card", if selected { Some("selected") } else { None })}>
                                                                <div class="exercise-head">
                                                                    <div>
                                                                        <h3>{ &exercise.nome }</h3>
                                                                        <div class="exercise-meta">{ format!("{} serie · {}", exercise.serie, exercise.reps) }</div>
                                                                    </div>
                                                                    <button class="select-button" onclick={onclick}>{ if selected { "Selezionato" } else { "Seleziona" } }</button>
                                                                </div>
                                                                <div class="exercise-rec">{ format!("Recupero: {}s", exercise.recupero.unwrap_or(90)) }</div>
                                                                if let Some(note) = &exercise.note {
                                                                    <p class="exercise-note">{ note.clone() }</p>
                                                                }
                                                                if selected {
                                                                    <div class="expanded-body">
                                                                        <div class="series-progress">
                                                                            { for (1..=exercise.serie).map(|serie_num| {
                                                                                let completed = (*saved_sets)
                                                                                    .iter()
                                                                                    .any(|item| item.exercise_id == exercise.id && item.set_number == serie_num);
                                                                                html! {
                                                                                    <span class={classes!("series-dot", if completed { Some("completed") } else { None })} title={format!("Serie {}", serie_num)}></span>
                                                                                }
                                                                            }) }
                                                                        </div>
                                                                        { for (0..exercise.serie as usize).map(|series_idx| {
                                                                            let set_number = (series_idx + 1) as u32;
                                                                            let completed = (*saved_sets)
                                                                                .iter()
                                                                                .any(|item| item.exercise_id == exercise.id && item.set_number == set_number);
                                                                            let exercise_id_clone = exercise_id.clone();
                                                                            let save_set = on_save_set.clone();
                                                                            let weight_value = weight_inputs
                                                                                .get(&exercise_id_clone)
                                                                                .and_then(|values| values.get(series_idx).cloned())
                                                                                .filter(|value| !value.is_empty())
                                                                                .or_else(|| {
                                                                                    (0..series_idx).rev().find_map(|prev_idx| {
                                                                                        weight_inputs
                                                                                            .get(&exercise_id_clone)
                                                                                            .and_then(|values| values.get(prev_idx).cloned())
                                                                                            .filter(|value| !value.is_empty())
                                                                                    })
                                                                                })
                                                                                .unwrap_or_default();
                                                                            let reps_value = reps_inputs
                                                                                .get(&exercise_id_clone)
                                                                                .and_then(|values| values.get(series_idx).cloned())
                                                                                .filter(|value| !value.is_empty())
                                                                                .or_else(|| {
                                                                                    (0..series_idx).rev().find_map(|prev_idx| {
                                                                                        reps_inputs
                                                                                            .get(&exercise_id_clone)
                                                                                            .and_then(|values| values.get(prev_idx).cloned())
                                                                                            .filter(|value| !value.is_empty())
                                                                                    })
                                                                                })
                                                                                .unwrap_or_else(|| exercise.reps.clone());
                                                                            html! {
                                                                                <div class="series-row">
                                                                                    <div class="series-row-header">
                                                                                        <span>{ format!("Serie {}", set_number) }</span>
                                                                                        { if completed {
                                                                                            html! { <span class="series-status">{"Completata"}</span> }
                                                                                        } else {
                                                                                            html! { <span class="series-status pending">{"In attesa"}</span> }
                                                                                        } }
                                                                                    </div>
                                                                                    <div class="input-row">
                                                                                        <label>
                                                                                            {"Peso (kg)"}
                                                                                            <input
                                                                                                value={weight_value}
                                                                                                placeholder="es. 80"
                                                                                                oninput={
                                                                                                    let on_weight_change = on_weight_change.clone();
                                                                                                    let exercise_id = exercise_id_clone.clone();
                                                                                                    Callback::from(move |e: InputEvent| {
                                                                                                        if let Some(input) = e.target().and_then(|t| t.dyn_into::<HtmlInputElement>().ok()) {
                                                                                                            on_weight_change.emit((exercise_id.clone(), series_idx, input.value()));
                                                                                                        }
                                                                                                    })
                                                                                                }
                                                                                            />
                                                                                        </label>
                                                                                        <label>
                                                                                            {"Reps"}
                                                                                            <input
                                                                                                value={reps_value}
                                                                                                placeholder={exercise.reps.clone()}
                                                                                                oninput={
                                                                                                    let on_reps_change = on_reps_change.clone();
                                                                                                    let exercise_id = exercise_id_clone.clone();
                                                                                                    Callback::from(move |e: InputEvent| {
                                                                                                        if let Some(input) = e.target().and_then(|t| t.dyn_into::<HtmlInputElement>().ok()) {
                                                                                                            on_reps_change.emit((exercise_id.clone(), series_idx, input.value()));
                                                                                                        }
                                                                                                    })
                                                                                                }
                                                                                            />
                                                                                        </label>
                                                                                        <button class="primary-button" onclick={Callback::from(move |_| save_set.emit(series_idx))}>
                                                                                            { if completed { "Aggiorna serie" } else { "Registra serie" } }
                                                                                        </button>
                                                                                    </div>
                                                                                </div>
                                                                            }
                                                                        }) }
                                                                        <div class="action-row">
                                                                            <button class="secondary-button" onclick={on_start_timer.clone()}>
                                                                                { if *timer_running { "Timer in corso" } else { "Avvia recupero" } }
                                                                            </button>
                                                                        </div>
                                                                        { if *timer_running {
                                                                            html! {
                                                                                <div class="timer-card">
                                                                                    <div class="timer-label">{"Recupero in corso"}</div>
                                                                                    <div class="timer-value">{ format!("{}s", *timer_left) }</div>
                                                                                    <div class="timer-bar"><div class="timer-bar-fill" style={format!("width:{}%", if *timer_total > 0 { (*timer_left as f32 / *timer_total as f32) * 100.0 } else { 0.0 })}></div></div>
                                                                                </div>
                                                                            }
                                                                        } else {
                                                                            html! {}
                                                                        } }
                                                                    </div>
                                                                }
                                                            </article>
                                                        }
                                                    }) }
                                                </section>

                                                <section class="session-history">
                                                    <h3>{"Storico serie"}</h3>
                                                    { if current_session.is_empty() {
                                                        html! { <p>{"Nessuna serie registrata per questo giorno."}</p> }
                                                    } else {
                                                        html! {
                                                            <ul>
                                                                { for current_session.iter().map(|entry| html! {
                                                                    <li>
                                                                        <strong>{ format!("{} - set {}", entry.nome, entry.set_number) }</strong>
                                                                        <div>{ format!("Peso: {} reps: {}", entry.peso.map(|v| v.to_string()).unwrap_or_else(|| "-".into()), entry.reps.clone().unwrap_or_else(|| "-".into())) }</div>
                                                                        <div class="history-time">{ &entry.timestamp }</div>
                                                                    </li>
                                                                }) }
                                                            </ul>
                                                        }
                                                    } }
                                                </section>
                                            </>
                                        }
                                    } else {
                                        html! { <p>{"Giorno non trovato."}</p> }
                                    }
                                }
                            </div>
                        }
                    } else {
                        html! {
                            <section class="upload-panel">
                                <div class="upload-card">
                                    <p>{"Scegli una scheda predefinita o carica un file JSON personale."}</p>
                                    { if *catalog_loading {
                                        html! { <p class="hint">{"Caricamento catalogo schede in corso..."}</p> }
                                    } else if catalog.is_empty() {
                                        html! { <p class="hint">{"Nessuna scheda disponibile nel catalogo."}</p> }
                                    } else {
                                        html! {
                                            <div class="catalog-list">
                                                { for catalog.iter().map(|entry| {
                                                    let item = entry.clone();
                                                    let on_load_catalog_entry = on_load_catalog_entry.clone();
                                                    html! {
                                                        <article class="catalog-card" onclick={Callback::from(move |_| on_load_catalog_entry.emit(item.clone()))}>
                                                            <div class="catalog-info">
                                                                <div class="catalog-title">{ format!("{} {}", entry.nome, entry.numero.clone().unwrap_or_default()) }</div>
                                                                <div class="catalog-meta">{ format!("{} / {}", entry.mese.clone().unwrap_or_default(), entry.anno.clone().unwrap_or_default()) }</div>
                                                            </div>
                                                            <button class="select-button">{"Apri"}</button>
                                                        </article>
                                                    }
                                                }) }
                                            </div>
                                        }
                                    }}
                                    <div style="margin-top: 24px;">
                                        <label class="file-label">
                                            <span>{"Carica file JSON"}</span>
                                            <input type="file" accept=".json" onchange={on_file_change} />
                                        </label>
                                    </div>
                                    <p class="hint">{"Il file deve contenere un oggetto JSON con campi: id, nome, giorni -> esercizi."}</p>
                                </div>
                            </section>
                        }
                    }
                }
                if let Some(error_msg) = &*error {
                    <div class="error-banner">{error_msg}</div>
                }
            </main>
        </div>
    }
}

#[wasm_bindgen(start)]
pub fn start() {
    // Improve panic messages and backtraces in browser console during development
    console_error_panic_hook::set_once();
    yew::Renderer::<App>::new().render();
}
