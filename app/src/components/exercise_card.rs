use crate::components::icons::icon_revert;
use crate::components::progress_bar::ProgressBar;
use crate::models::{parse_reps_range, CompletedSet, Exercise};
use gloo_timers::callback::Timeout;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct ExerciseCardProps {
    pub exercise:      Exercise,
    pub saved_sets:    Vec<CompletedSet>,
    pub is_selected:   bool,
    pub is_overridden: bool,
    pub on_select:     Callback<()>,
    /// Called after a 400ms long-press on the exercise name — opens the picker.
    pub on_long_press: Callback<()>,
    /// Called when the user taps "↩ Ripristina" on an overridden exercise.
    pub on_revert:     Callback<()>,
}

#[function_component(ExerciseCard)]
pub fn exercise_card(props: &ExerciseCardProps) -> Html {
    let exercise = &props.exercise;
    let n        = exercise.serie as usize;
    let lp_timeout = use_mut_ref(|| None::<Timeout>);

    let dot_done: Vec<bool> = (0..n).map(|i| {
        let s = (i + 1) as u32;
        props.saved_sets.iter()
            .any(|e| e.exercise_id == exercise.id && e.set_number == s)
    }).collect();

    let active_hint = dot_done.iter().position(|&done| !done).unwrap_or(0);

    let (reps_min, reps_max) = parse_reps_range(&exercise.reps);
    let dot_reps_hint: Vec<Option<i8>> = (0..n).map(|i| {
        let s = (i + 1) as u32;
        props.saved_sets.iter()
            .find(|e| e.exercise_id == exercise.id && e.set_number == s)
            .and_then(|set| set.reps.as_ref())
            .and_then(|r| r.parse::<i32>().ok())
            .map(|actual| {
                if reps_min > 0 && actual < reps_min { -1i8 }
                else if reps_max > 0 && actual > reps_max { 1i8 }
                else { 0i8 }
            })
    }).collect();

    let onclick_card = {
        let on_select = props.on_select.clone();
        Callback::from(move |_: MouseEvent| on_select.emit(()))
    };

    // ── Long-press on the name: 400ms hold → open picker ─────────────────────
    let on_name_pointerdown = {
        let cb  = props.on_long_press.clone();
        let lpt = lp_timeout.clone();
        Callback::from(move |e: PointerEvent| {
            e.stop_propagation();
            let cb2 = cb.clone();
            let t   = Timeout::new(400, move || cb2.emit(()));
            *lpt.borrow_mut() = Some(t);
        })
    };
    let on_name_pointerup = {
        let lpt = lp_timeout.clone();
        Callback::from(move |_: PointerEvent| { lpt.borrow_mut().take(); })
    };
    let on_name_pointercancel = {
        let lpt = lp_timeout.clone();
        Callback::from(move |_: PointerEvent| { lpt.borrow_mut().take(); })
    };

    let is_cardio = exercise.tipo.as_deref() == Some("cardio");

    let cardio_done_mins: Option<u32> = if is_cardio {
        props.saved_sets.iter()
            .find(|s| s.exercise_id == exercise.id && s.set_number == 1)
            .and_then(|s| s.durata_min)
    } else { None };

    html! {
        <article
            class={classes!("exercise-card", if props.is_selected { Some("selected") } else { None })}
            onclick={onclick_card}
        >
            <div class="exercise-head">
                <div style="flex:1;min-width:0;">
                    <div class="exercise-name-row">
                        <h3
                            onpointerdown={on_name_pointerdown}
                            onpointerup={on_name_pointerup}
                            onpointercancel={on_name_pointercancel}
                            style="user-select:none;-webkit-user-select:none;touch-action:none;"
                        >{ exercise.display_name() }</h3>
                        if props.is_overridden {
                            <button class="revert-btn"
                                onclick={{
                                    let cb = props.on_revert.clone();
                                    Callback::from(move |e: MouseEvent| {
                                        e.stop_propagation();
                                        cb.emit(());
                                    })
                                }}
                                title="Ripristina esercizio originale">
                                { icon_revert() }
                            </button>
                        }
                    </div>
                    <div class="exercise-meta">
                        if is_cardio {
                            { format!("Cardio · {}", exercise.reps) }
                        } else if exercise.tipo.as_deref() == Some("temporale") {
                            { format!("Isometrico · {}", exercise.reps) }
                        } else {
                            { format!("{} serie · {}", exercise.serie, exercise.reps) }
                        }
                    </div>
                </div>
            </div>
            if !is_cardio {
                if exercise.recupero.is_some() {
                    <div class="exercise-rec">
                        { format!("Recupero: {}s", exercise.recupero.unwrap_or(0)) }
                    </div>
                }
            }
            <ProgressBar
                n={exercise.serie}
                dot_done={dot_done}
                dot_reps_hint={dot_reps_hint}
                active={active_hint}
                on_select={{
                    let on_sel = props.on_select.clone();
                    Callback::from(move |_: usize| on_sel.emit(()))
                }}
            />
            if let Some(mins) = cardio_done_mins {
                <div class="cardio-done-tag">{ format!("{}min registrati", mins) }</div>
            }
        </article>
    }
}
