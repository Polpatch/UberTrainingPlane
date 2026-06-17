use crate::models::ExerciseDef;
use wasm_bindgen::JsCast;
use web_sys::HtmlInputElement;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct ExercisePickerProps {
    /// Full exercise library sorted by nome.
    pub library:   Vec<ExerciseDef>,
    /// Emits the chosen exercise id.
    pub on_select: Callback<String>,
    pub on_cancel: Callback<()>,
}

#[function_component(ExercisePicker)]
pub fn exercise_picker(props: &ExercisePickerProps) -> Html {
    let query    = use_state(String::new);
    let selected = use_state(|| None::<String>);

    let q = (*query).to_lowercase();
    let filtered: Vec<&ExerciseDef> = props.library.iter()
        .filter(|e| q.is_empty() || e.nome.to_lowercase().contains(&q))
        .collect();

    html! {
        <div class="picker-overlay"
             onclick={{ let cb = props.on_cancel.clone();
                        Callback::from(move |_: MouseEvent| cb.emit(())) }}>

            <div class="picker-modal"
                 onclick={Callback::from(|e: MouseEvent| e.stop_propagation())}>

                // ── Header ───────────────────────────────────────────────
                <div class="picker-header">
                    <span class="picker-title">{"Esercizio alternativo"}</span>
                    <button class="menu-close-btn"
                        onclick={{ let cb = props.on_cancel.clone();
                                   Callback::from(move |_: MouseEvent| cb.emit(())) }}>
                        {"✕"}
                    </button>
                </div>

                // ── Search ───────────────────────────────────────────────
                <input
                    class="picker-search"
                    type="text"
                    placeholder="Cerca esercizio…"
                    value={(*query).clone()}
                    oninput={{
                        let q = query.clone();
                        Callback::from(move |e: InputEvent| {
                            if let Some(el) = e.target()
                                .and_then(|t| t.dyn_into::<HtmlInputElement>().ok())
                            {
                                q.set(el.value());
                            }
                        })
                    }}
                />

                // ── List ─────────────────────────────────────────────────
                <div class="picker-list">
                    { for filtered.iter().map(|def| {
                        let id       = def.id.clone();
                        let nome     = def.nome.clone();
                        let tipo_str = def.tipo.clone();
                        let is_sel   = selected.as_deref() == Some(&def.id);
                        let sel      = selected.clone();
                        html! {
                            <button
                                class={classes!(
                                    "picker-item",
                                    if is_sel { Some("picker-item--selected") } else { None }
                                )}
                                onclick={{ let id = id.clone();
                                           Callback::from(move |_: MouseEvent| sel.set(Some(id.clone()))) }}
                            >
                                <span class="picker-item-nome">{ nome }</span>
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
