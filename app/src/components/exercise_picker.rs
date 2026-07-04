use crate::models::ExerciseDef;
use wasm_bindgen::JsCast;
use web_sys::{HtmlInputElement, HtmlSelectElement};
use yew::prelude::*;

/// Muscle groups shown in the filter dropdown.
/// Keys match the DB's `primaryMuscles`/`secondaryMuscles` strings.
const MUSCLE_OPTIONS: &[(&str, &str)] = &[
    ("chest",       "Petto"),
    ("shoulders",   "Spalle"),
    ("lats",        "Gran dorsale"),
    ("middle back", "Dorsale medio"),
    ("lower back",  "Lombare"),
    ("biceps",      "Bicipiti"),
    ("triceps",     "Tricipiti"),
    ("abdominals",  "Addominali"),
    ("quadriceps",  "Quadricipiti"),
    ("hamstrings",  "Femorali"),
    ("glutes",      "Glutei"),
    ("calves",      "Polpacci"),
    ("traps",       "Trapezi"),
    ("forearms",    "Avambracci"),
    ("adductors",   "Adduttori"),
    ("abductors",   "Abduttori"),
];

/// Equipment categories shown in the filter dropdown.
const EQUIP_OPTIONS: &[(&str, &str)] = &[
    ("barbell",      "Bilanciere"),
    ("dumbbell",     "Manubri"),
    ("cable",        "Cavi"),
    ("machine",      "Macchina"),
    ("body only",    "Corpo libero"),
    ("kettlebells",  "Kettlebell"),
    ("bands",        "Elastici"),
    ("e-z curl bar", "EZ Bar"),
];

fn muscle_it(key: &str) -> &str {
    MUSCLE_OPTIONS.iter().find(|(k, _)| *k == key).map(|(_, v)| *v).unwrap_or(key)
}

fn equip_it(key: &str) -> &str {
    EQUIP_OPTIONS.iter().find(|(k, _)| *k == key).map(|(_, v)| *v).unwrap_or(key)
}

#[derive(Properties, PartialEq)]
pub struct ExercisePickerProps {
    /// Full exercise library sorted by display name.
    pub library:   Vec<ExerciseDef>,
    /// Emits the chosen exercise id.
    pub on_select: Callback<String>,
    pub on_cancel: Callback<()>,
}

#[function_component(ExercisePicker)]
pub fn exercise_picker(props: &ExercisePickerProps) -> Html {
    let query         = use_state(String::new);
    let selected      = use_state(|| None::<String>);
    let muscle_filter = use_state(|| None::<String>);
    let equip_filter  = use_state(|| None::<String>);

    let q = (*query).to_lowercase();

    let filtered: Vec<&ExerciseDef> = props.library.iter()
        .filter(|e| {
            let text_ok = q.is_empty()
                || e.display_name().to_lowercase().contains(&q)
                || e.name.to_lowercase().contains(&q);
            let muscle_ok = muscle_filter.as_ref().map_or(true, |m| {
                e.primary_muscles.iter().any(|pm| pm == m)
                    || e.secondary_muscles.iter().any(|sm| sm == m)
            });
            let equip_ok = equip_filter.as_ref().map_or(true, |eq| {
                e.equipment.as_deref() == Some(eq.as_str())
            });
            text_ok && muscle_ok && equip_ok
        })
        .collect();

    // Dropdown change callbacks — also clear selection since the item may leave the list.
    let on_muscle_change = {
        let mf  = muscle_filter.clone();
        let sel = selected.clone();
        Callback::from(move |e: Event| {
            if let Some(el) = e.target().and_then(|t| t.dyn_into::<HtmlSelectElement>().ok()) {
                let v = el.value();
                mf.set(if v.is_empty() { None } else { Some(v) });
                sel.set(None);
            }
        })
    };
    let on_equip_change = {
        let ef  = equip_filter.clone();
        let sel = selected.clone();
        Callback::from(move |e: Event| {
            if let Some(el) = e.target().and_then(|t| t.dyn_into::<HtmlSelectElement>().ok()) {
                let v = el.value();
                ef.set(if v.is_empty() { None } else { Some(v) });
                sel.set(None);
            }
        })
    };

    html! {
        <div class="picker-overlay"
             tabindex="0"
             onclick={{ let cb = props.on_cancel.clone();
                        Callback::from(move |_: MouseEvent| cb.emit(())) }}
             onkeydown={{
                 let cb = props.on_cancel.clone();
                 Callback::from(move |e: KeyboardEvent| {
                     if e.key() == "Escape" {
                         e.prevent_default();
                         cb.emit(());
                     }
                 })
             }}>

            <div class="picker-modal"
                 role="dialog"
                 aria-modal="true"
                 aria-labelledby="exercise-picker-title"
                 tabindex="-1"
                 onclick={Callback::from(|e: MouseEvent| e.stop_propagation())}>

                // ── Header ───────────────────────────────────────────────
                <div class="picker-header">
                    <span id="exercise-picker-title" class="picker-title">{"Esercizio alternativo"}</span>
                    <button class="menu-close-btn"
                        aria-label="Chiudi selezione esercizio"
                        onclick={{ let cb = props.on_cancel.clone();
                                   Callback::from(move |_: MouseEvent| cb.emit(())) }}>
                        {"✕"}
                    </button>
                </div>

                // ── Search ───────────────────────────────────────────────
                <input
                    class="picker-search"
                    type="text"
                    placeholder="Cerca per nome (IT o EN)…"
                    autofocus={true}
                    value={(*query).clone()}
                    oninput={{
                        let q   = query.clone();
                        let sel = selected.clone();
                        Callback::from(move |e: InputEvent| {
                            if let Some(el) = e.target()
                                .and_then(|t| t.dyn_into::<HtmlInputElement>().ok())
                            {
                                q.set(el.value());
                                sel.set(None);
                            }
                        })
                    }}
                />

                // ── Filter dropdowns ─────────────────────────────────────
                <div class="picker-filters">
                    <select class="picker-select" onchange={on_muscle_change}>
                        <option value="" selected={muscle_filter.is_none()}>{"Tutti i muscoli"}</option>
                        { for MUSCLE_OPTIONS.iter().map(|(key, label)| {
                            let is_sel = muscle_filter.as_deref() == Some(*key);
                            html! { <option value={*key} selected={is_sel}>{ *label }</option> }
                        }) }
                    </select>
                    <select class="picker-select" onchange={on_equip_change}>
                        <option value="" selected={equip_filter.is_none()}>{"Tutti gli attrezzi"}</option>
                        { for EQUIP_OPTIONS.iter().map(|(key, label)| {
                            let is_sel = equip_filter.as_deref() == Some(*key);
                            html! { <option value={*key} selected={is_sel}>{ *label }</option> }
                        }) }
                    </select>
                </div>

                // ── Result count ─────────────────────────────────────────
                <p class="picker-count">
                    { format!("{} esercizi", filtered.len()) }
                </p>

                // ── List ─────────────────────────────────────────────────
                <div class="picker-list">
                    { for filtered.iter().map(|def| {
                        let id      = def.id.clone();
                        let nome    = def.display_name().to_string();
                        let tipo_str = def.tipo().to_string();
                        let is_sel  = selected.as_deref() == Some(&def.id);
                        let sel     = selected.clone();

                        // Subtitle: attrezzo · primo muscolo primario (translated)
                        let subtitle = {
                            let mut parts: Vec<&str> = Vec::new();
                            if let Some(eq) = def.equipment.as_deref() {
                                parts.push(equip_it(eq));
                            }
                            if let Some(m) = def.primary_muscles.first() {
                                parts.push(muscle_it(m));
                            }
                            parts.join(" · ")
                        };

                        html! {
                            <button
                                key={id.clone()}
                                class={classes!(
                                    "picker-item",
                                    is_sel.then_some("picker-item--selected")
                                )}
                                onclick={{ let id = id.clone();
                                           Callback::from(move |_: MouseEvent| sel.set(Some(id.clone()))) }}
                            >
                                <div class="picker-item-body">
                                    <span class="picker-item-nome">{ nome }</span>
                                    if !subtitle.is_empty() {
                                        <span class="picker-item-sub">{ subtitle }</span>
                                    }
                                </div>
                                <span class="picker-item-tipo">{ tipo_str }</span>
                            </button>
                        }
                    }) }
                    if filtered.is_empty() {
                        <p class="picker-empty">{"Nessun esercizio trovato."}</p>
                    }
                </div>

                // ── Footer ───────────────────────────────────────────────
                <div class="picker-footer">
                    <button class="secondary-button"
                        onclick={{ let cb = props.on_cancel.clone();
                                   Callback::from(move |_: MouseEvent| cb.emit(())) }}>
                        {"Annulla"}
                    </button>
                    <button class="primary-button"
                        disabled={selected.is_none()}
                        onclick={{
                            let sel = selected.clone();
                            let cb  = props.on_select.clone();
                            Callback::from(move |_: MouseEvent| {
                                if let Some(id) = (*sel).clone() { cb.emit(id); }
                            })
                        }}>
                        {"Seleziona"}
                    </button>
                </div>
            </div>
        </div>
    }
}
