use crate::components::icons::{icon_chart, icon_photo, icon_play};
use crate::components::progress_bar::ProgressBar;
use crate::models::{
    get_input_with_fallback, image_url, last_reps_for_exercise_set, parse_reps_range,
    weight_history_for_exercise, CompletedSet, Day, Exercise, TimerState, WeightPoint,
};
use gloo_timers::callback::Timeout;
use std::collections::HashMap;
use wasm_bindgen::JsCast;
use web_sys::HtmlInputElement;
use yew::prelude::*;

const STEP_VALUES: [f32; 5] = [0.5, 1.0, 2.0, 5.0, 10.0];


/// Convert any YouTube URL format to an embeddable URL with autoplay.
fn youtube_embed_url(url: &str) -> Option<String> {
    if url.contains("youtube.com/embed/") {
        return Some(format!("{}&autoplay=1&rel=0&modestbranding=1", url));
    }
    let id = if let Some(pos) = url.find("v=") {
        url[pos + 2..].split('&').next()?
    } else if url.contains("youtu.be/") {
        url.split("youtu.be/").nth(1)?.split('?').next()?
    } else if url.contains("shorts/") {
        url.split("shorts/").nth(1)?.split('?').next()?
    } else {
        return None;
    };
    Some(format!("https://www.youtube.com/embed/{}?autoplay=1&rel=0&modestbranding=1", id))
}

/// Trigger a short haptic pulse (Android only; silently ignored elsewhere).
fn vibrate(ms: u32) {
    let Some(window) = web_sys::window() else { return };
    let nav = window.navigator();
    let Ok(vib) = js_sys::Reflect::get(&nav, &"vibrate".into()) else { return };
    if let Some(f) = vib.dyn_ref::<js_sys::Function>() {
        let _ = f.call1(&nav, &wasm_bindgen::JsValue::from_f64(ms as f64));
    }
}

/// Dismiss the soft keyboard by blurring the currently focused element.
fn blur_active() {
    let Some(window)   = web_sys::window()   else { return };
    let Some(document) = window.document()   else { return };
    let Some(active)   = document.active_element() else { return };
    let Ok(blur) = js_sys::Reflect::get(&active, &"blur".into()) else { return };
    if let Some(f) = blur.dyn_ref::<js_sys::Function>() {
        let _ = f.call0(&active);
    }
}

fn fmt_weight(w: f32) -> String {
    if w.fract() == 0.0 { format!("{:.0}", w) } else { format!("{:.1}", w) }
}

fn render_weight_chart(points: &[WeightPoint]) -> Html {
    if points.is_empty() {
        return html! {
            <p class="chart-empty">
                {"Nessun dato ancora. Completa almeno un allenamento per vedere il grafico."}
            </p>
        };
    }
    let view_width:  f64 = 300.0;
    let view_height: f64 = 110.0;
    let pad_left:    f64 = 32.0;
    let pad_right:   f64 = 10.0;
    let pad_top:     f64 = 18.0;
    let pad_bottom:  f64 = 24.0;
    let inner_w = view_width  - pad_left  - pad_right;
    let inner_h = view_height - pad_top   - pad_bottom;
    let n = points.len();
    let min_w = points.iter().map(|p| p.max_weight).fold(f32::INFINITY,     f32::min);
    let max_w = points.iter().map(|p| p.max_weight).fold(f32::NEG_INFINITY, f32::max);
    let weight_range = ((max_w - min_w) as f64).max(1.0);
    let to_x = |i: usize| -> f64 {
        pad_left + if n <= 1 { inner_w / 2.0 } else { i as f64 / (n - 1) as f64 * inner_w }
    };
    let to_y = |w: f32| -> f64 {
        pad_top + inner_h - (w as f64 - min_w as f64) / weight_range * inner_h
    };
    let path_d: String = points.iter().enumerate()
        .map(|(i, p)| {
            let x = to_x(i); let y = to_y(p.max_weight);
            if i == 0 { format!("M {x:.1} {y:.1}") } else { format!("L {x:.1} {y:.1}") }
        })
        .collect::<Vec<_>>().join(" ");
    let label_max_w  = format!("{:.0}kg", max_w);
    let label_min_w  = format!("{:.0}kg", min_w);
    let y_top_str    = format!("{:.1}", pad_top);
    let y_bot_str    = format!("{:.1}", pad_top + inner_h);
    let x_ylabel_str = format!("{:.0}", pad_left - 4.0);
    html! {
        <svg viewBox={format!("0 0 {view_width} {view_height}")} width="100%" height="130"
             style="display:block;overflow:visible">
            <line class="chart-gridline-light"
                  x1={format!("{pad_left}")} y1={y_top_str.clone()}
                  x2={format!("{:.0}", view_width - pad_right)} y2={y_top_str.clone()}
                  stroke="#f3f4f6" stroke-width="1"/>
            <line class="chart-gridline"
                  x1={format!("{pad_left}")} y1={y_bot_str.clone()}
                  x2={format!("{:.0}", view_width - pad_right)} y2={y_bot_str.clone()}
                  stroke="#e5e7eb" stroke-width="1"/>
            <text class="chart-ylabel" x={x_ylabel_str.clone()} y={y_top_str}
                  text-anchor="end" font-size="9" fill="#9ca3af">{ label_max_w }</text>
            <text class="chart-ylabel" x={x_ylabel_str} y={format!("{:.1}", pad_top + inner_h + 3.0)}
                  text-anchor="end" font-size="9" fill="#9ca3af">{ label_min_w }</text>
            <path class="chart-line" d={path_d} fill="none" stroke="#2563eb"
                  stroke-width="2" stroke-linejoin="round" stroke-linecap="round"/>
            { for points.iter().enumerate().map(|(i, p)| {
                let x = to_x(i); let y = to_y(p.max_weight);
                let show_date = i == 0 || i == n - 1;
                let date_str  = p.date.get(5..).unwrap_or(&p.date).to_string();
                let anchor    = if i == 0 { "start" } else { "end" };
                let dy        = format!("{:.1}", view_height - 4.0);
                html! {
                    <g>
                        <circle class="chart-dot"
                                cx={format!("{x:.1}")} cy={format!("{y:.1}")} r="3.5"
                                fill="#2563eb" stroke="white" stroke-width="1.5"/>
                        <text class="chart-value"
                              x={format!("{x:.1}")} y={format!("{:.1}", y - 7.0)}
                              text-anchor="middle" font-size="9" fill="#111" font-weight="600">
                            { format!("{:.1}", p.max_weight) }
                        </text>
                        { if show_date { html! {
                            <text class="chart-date"
                                  x={format!("{x:.1}")} y={dy}
                                  text-anchor={anchor} font-size="9" fill="#9ca3af">
                                { date_str }
                            </text>
                        }} else { html! {} } }
                    </g>
                }
            }) }
        </svg>
    }
}

#[derive(Properties, PartialEq)]
pub struct BottomSheetProps {
    /// Currently selected exercise — None hides the sheet.
    pub exercise:         Option<Exercise>,
    /// Current day (needed for exercise-level auto-advance).
    pub day:              Option<Day>,
    pub saved_sets:       Vec<CompletedSet>,
    pub weight_inputs:    HashMap<String, Vec<String>>,
    pub reps_inputs:      HashMap<String, Vec<String>>,
    pub on_save_set:      Callback<usize>,
    pub on_weight_change: Callback<(String, usize, String)>,
    pub on_reps_change:   Callback<(String, usize, String)>,
    pub on_start_timer:   Callback<()>,
    pub on_cancel_timer:  Callback<()>,
    pub timer:            TimerState,
    pub history_mode:     bool,
    pub workout_id:       String,
    /// Index of the selected exercise in the day (for exercise-level auto-advance).
    pub selected_exercise_idx: usize,
    pub on_select_exercise:    Callback<usize>,
    /// Increments every time the user taps an exercise card — forces sheet open.
    pub expand_trigger: usize,
    /// Cardio stopwatch state (seconds elapsed, running flag, callbacks).
    pub cardio_elapsed:   u32,
    pub cardio_running:   bool,
    pub on_cardio_toggle: Callback<()>,
    pub on_cardio_stop:   Callback<()>,
    /// Timer for exercises of tipo=="temporale" (countdown from exercise.durata).
    pub timed_timer:      TimerState,
    pub on_timed_toggle:  Callback<()>,
    pub on_timed_stop:    Callback<()>,
    /// Called whenever the sheet expands or collapses — lets the parent adjust layout.
    pub on_expand_change: Callback<bool>,
    /// Right-handed layout — primary action on the right. From UserPreferences.
    pub righthanded: bool,
}

#[function_component(BottomSheet)]
pub fn bottom_sheet(props: &BottomSheetProps) -> Html {
    let active_set         = use_state(|| 0usize);
    let step_idx           = use_state(|| 1usize); // 1.0 kg default
    let chart_open         = use_state(|| false);
    let expanded           = use_state(|| false);
    let video_open         = use_state(|| false);
    let photo_open         = use_state(|| false);
    let photo_idx          = use_state(|| 0usize);
    let just_saved         = use_state(|| None::<usize>);
    let just_saved_timeout = use_mut_ref(|| None::<Timeout>);
    // Swipe-to-expand/collapse: track Y at pointerdown on the handle.
    let drag_start_y = use_mut_ref(|| 0i32);
    let drag_moved   = use_mut_ref(|| false);

    // ── Values computed before hooks (hooks must run unconditionally) ─────────
    let exercise_id = props.exercise.as_ref().map(|e| e.id.clone()).unwrap_or_default();
    let n           = props.exercise.as_ref().map(|e| e.serie as usize).unwrap_or(0);
    let clamped     = (*active_set).min(n.saturating_sub(1));

    let dot_done: Vec<bool> = (0..n).map(|i| {
        let s = (i + 1) as u32;
        props.saved_sets.iter()
            .any(|e| e.exercise_id == exercise_id && e.set_number == s)
    }).collect();

    let (reps_min, reps_max) = props.exercise.as_ref()
        .map(|e| parse_reps_range(&e.reps))
        .unwrap_or((0, 0));
    let dot_reps_hint: Vec<Option<i8>> = (0..n).map(|i| {
        let s = (i + 1) as u32;
        props.saved_sets.iter()
            .find(|e| e.exercise_id == exercise_id && e.set_number == s)
            .and_then(|set| set.reps.as_ref())
            .and_then(|r| r.parse::<i32>().ok())
            .map(|actual| {
                if reps_min > 0 && actual < reps_min { -1i8 }
                else if reps_max > 0 && actual > reps_max { 1i8 }
                else { 0i8 }
            })
    }).collect();

    let n_saves = props.saved_sets.iter()
        .filter(|s| s.exercise_id == exercise_id)
        .count();

    let completed_count = dot_done.iter().filter(|&&d| d).count();

    // ── Hook: reset active_set when exercise changes ──────────────────────────
    {
        let asc      = active_set.clone();
        let saved    = props.saved_sets.clone();
        let eid      = exercise_id.clone();
        let n_effect = n;
        use_effect_with_deps(
            move |_id: &String| {
                let first = (0..n_effect).find(|&i| {
                    let s = (i + 1) as u32;
                    !saved.iter().any(|set| set.exercise_id == eid && set.set_number == s)
                }).unwrap_or(0);
                asc.set(first);
                || ()
            },
            exercise_id.clone(),
        );
    }

    // ── Hook: auto-advance active_set after timer saves a set ─────────────────
    {
        let asc  = active_set.clone();
        let snap = dot_done.clone();
        let cv   = clamped;
        use_effect_with_deps(
            move |_: &usize| {
                if snap.get(cv).copied().unwrap_or(false) {
                    let next = (1..snap.len())
                        .map(|off| (cv + off) % snap.len())
                        .find(|&i| !snap.get(i).copied().unwrap_or(false));
                    if let Some(idx) = next { asc.set(idx); }
                }
                || ()
            },
            n_saves,
        );
    }

    // ── Hook: expand sheet on explicit card tap (trigger > 0 skips first render) ──
    {
        let exp = expanded.clone();
        use_effect_with_deps(
            move |trigger: &usize| {
                if *trigger > 0 { exp.set(true); }
                || ()
            },
            props.expand_trigger,
        );
    }

    // ── Hook: notify parent when sheet expands/collapses (for layout padding) ──
    {
        let cb = props.on_expand_change.clone();
        use_effect_with_deps(
            move |is_expanded: &bool| {
                cb.emit(*is_expanded);
                || ()
            },
            *expanded,
        );
    }

    // ── Early return when no exercise ─────────────────────────────────────────
    let exercise = match &props.exercise { Some(e) => e, None => return html! {} };

    let is_cardio   = exercise.tipo.as_deref() == Some("cardio");
    let is_timed    = exercise.tipo.as_deref() == Some("temporale");
    let cardio_done = completed_count > 0;

    // ── Input values ──────────────────────────────────────────────────────────
    // `_display` = raw stored value, may be empty — used for the input's value= prop
    //   so the user can freely clear the field without it snapping back.
    // `_value`   = with fallback — used only for calculations (+/- buttons, saving).
    let weight_display = props.weight_inputs.get(&exercise_id)
        .and_then(|v| v.get(clamped)).cloned().unwrap_or_default();
    let reps_display   = props.reps_inputs.get(&exercise_id)
        .and_then(|v| v.get(clamped)).cloned().unwrap_or_default();

    let weight_value = get_input_with_fallback(&props.weight_inputs, &exercise_id, clamped, "");
    let reps_value   = get_input_with_fallback(&props.reps_inputs,   &exercise_id, clamped, &exercise.reps);

    let set_number = (clamped + 1) as u32;

    // Hint from last session — shown as placeholder when no weight/reps entered yet
    let weight_hint: String = if weight_display.is_empty() {
        weight_history_for_exercise(&props.workout_id, &exercise.id)
            .last()
            .map(|p| fmt_weight(p.max_weight))
            .unwrap_or_default()
    } else {
        String::new()
    };
    let reps_hint: String = if reps_display.is_empty() {
        last_reps_for_exercise_set(&props.workout_id, &exercise.id, set_number)
            .unwrap_or_default()
    } else {
        String::new()
    };
    let completed = props.saved_sets.iter()
        .any(|s| s.exercise_id == exercise.id && s.set_number == set_number);

    // ── Step controls ─────────────────────────────────────────────────────────
    let step = STEP_VALUES[*step_idx];

    let on_cycle_step = {
        let si = step_idx.clone();
        Callback::from(move |e: MouseEvent| {
            e.stop_propagation();
            si.set((*si + 1) % STEP_VALUES.len());
        })
    };

    // When weight field is empty, start +/- from last session's value (the hint).
    let weight_f: f32 = weight_value.parse()
        .unwrap_or_else(|_| weight_hint.parse().unwrap_or(0.0));
    let on_weight_minus = {
        let cb = props.on_weight_change.clone(); let eid = exercise_id.clone();
        let val = fmt_weight((weight_f - step).max(0.0));
        Callback::from(move |_: MouseEvent| cb.emit((eid.clone(), clamped, val.clone())))
    };
    let on_weight_plus = {
        let cb = props.on_weight_change.clone(); let eid = exercise_id.clone();
        let val = fmt_weight(weight_f + step);
        Callback::from(move |_: MouseEvent| cb.emit((eid.clone(), clamped, val.clone())))
    };

    // When reps field holds a range like "8-10" (the exercise default), use the
    // lower bound so +/- starts from a sensible value instead of 1.
    // Prefer the last-session hint over the scheda's target range when empty.
    let reps_n: i32 = reps_value.parse()
        .unwrap_or_else(|_| {
            reps_hint.parse().ok()
                .unwrap_or_else(|| parse_reps_range(&reps_value).0.max(1))
        });
    let on_reps_minus = {
        let cb = props.on_reps_change.clone(); let eid = exercise_id.clone();
        let val = (reps_n - 1).max(1).to_string();
        Callback::from(move |_: MouseEvent| cb.emit((eid.clone(), clamped, val.clone())))
    };
    let on_reps_plus = {
        let cb = props.on_reps_change.clone(); let eid = exercise_id.clone();
        let val = (reps_n + 1).to_string();
        Callback::from(move |_: MouseEvent| cb.emit((eid.clone(), clamped, val.clone())))
    };

    // ── Save set + active_set advance + just-saved pulse ─────────────────────
    let on_register = {
        let save           = props.on_save_set.clone();
        let asc            = active_set.clone();
        let dot_snap       = dot_done.clone();
        let cancel_timer   = props.on_cancel_timer.clone();
        let stop_timed     = props.on_timed_stop.clone();
        let was_completed  = completed;
        let timer_active   = props.timer.running;
        let timed_active   = props.timed_timer.running || props.timed_timer.left > 0;
        let js             = just_saved.clone();
        let jst            = just_saved_timeout.clone();
        Callback::from(move |_: MouseEvent| {
            save.emit(clamped);
            vibrate(50);
            blur_active();
            // Trigger pulse animation on the saved dot
            js.set(Some(clamped));
            let js2 = js.clone();
            let t = Timeout::new(600, move || { js2.set(None); });
            *jst.borrow_mut() = Some(t);
            if !was_completed {
                if timer_active  { cancel_timer.emit(()); }
                if timed_active  { stop_timed.emit(());  }
                let next = (1..n)
                    .map(|off| (clamped + off) % n)
                    .find(|&i| !dot_snap.get(i).copied().unwrap_or(false));
                if let Some(idx) = next { asc.set(idx); }
            }
        })
    };

    // Chart data (lazy)
    let chart_points: Vec<WeightPoint> = if *chart_open {
        weight_history_for_exercise(&props.workout_id, &exercise.id)
    } else { vec![] };

    // Video embed URL (None if no video or unrecognised URL)
    let video_embed: Option<String> = exercise.video.as_deref().and_then(youtube_embed_url);

    // Photo carousel: current image URL + count, clamped to bounds (None if no images)
    let n_photos = exercise.images.len();
    let photo_clamped = (*photo_idx).min(n_photos.saturating_sub(1));
    let current_photo_url: Option<String> = exercise.images.get(photo_clamped).map(|p| image_url(p));

    let sheet_class = if *expanded { "bottom-sheet bottom-sheet--expanded" }
                      else         { "bottom-sheet bottom-sheet--minimized" };

    let on_handle_keydown = {
        let exp = expanded.clone();
        Callback::from(move |e: KeyboardEvent| {
            let key = e.key();
            if key == "Enter" || key == " " {
                e.prevent_default();
                exp.set(!*exp);
            }
        })
    };

    let close_video = {
        let vo = video_open.clone();
        Callback::from(move |_: MouseEvent| vo.set(false))
    };
    let close_photo = {
        let po = photo_open.clone();
        Callback::from(move |_: MouseEvent| po.set(false))
    };
    let close_video_on_escape = {
        let vo = video_open.clone();
        Callback::from(move |e: KeyboardEvent| {
            if e.key() == "Escape" {
                e.prevent_default();
                vo.set(false);
            }
        })
    };
    let close_photo_on_escape = {
        let po = photo_open.clone();
        Callback::from(move |e: KeyboardEvent| {
            if e.key() == "Escape" {
                e.prevent_default();
                po.set(false);
            }
        })
    };

    html! {
        <>
        <div class={sheet_class}>
            // ── Backdrop — collapses sheet on outside tap ─────────────────
            if *expanded {
                <div class="sheet-backdrop"
                     onclick={{ let exp = expanded.clone(); Callback::from(move |_: MouseEvent| exp.set(false)) }}>
                </div>
            }
            // ── Handle area (always visible) ──────────────────────────────
            <div class="sheet-handle-area"
                 role="button"
                 tabindex="0"
                 aria-expanded={expanded.to_string()}
                 aria-label={format!(
                     "{} dettagli esercizio {}",
                     if *expanded { "Comprimi" } else { "Espandi" },
                     exercise.display_name(),
                 )}
                 onkeydown={on_handle_keydown}
                 onpointerdown={{
                     let dsy  = drag_start_y.clone();
                     let dmov = drag_moved.clone();
                     Callback::from(move |e: PointerEvent| {
                         *dsy.borrow_mut()  = e.client_y();
                         *dmov.borrow_mut() = false;
                     })
                 }}
                 onpointermove={{
                     let dsy  = drag_start_y.clone();
                     let dmov = drag_moved.clone();
                     Callback::from(move |e: PointerEvent| {
                         if (e.client_y() - *dsy.borrow()).abs() > 8 {
                             *dmov.borrow_mut() = true;
                         }
                     })
                 }}
                 onpointerup={{
                     let exp  = expanded.clone();
                     let dsy  = drag_start_y.clone();
                     let dmov = drag_moved.clone();
                     Callback::from(move |e: PointerEvent| {
                         let dy    = e.client_y() - *dsy.borrow();
                         let moved = *dmov.borrow();
                         if moved {
                             if dy < -30      { exp.set(true); }
                             else if dy > 30  { exp.set(false); }
                         } else {
                             exp.set(!*exp);
                         }
                     })
                 }}>
                <div class="sheet-handle-pill"></div>
                <span class="sheet-ex-name">{ exercise.display_name() }</span>
                <span class="sheet-progress-mini">
                    if is_cardio {
                        {
                            if props.cardio_running || props.cardio_elapsed > 0 {
                                format!("{:02}:{:02}", props.cardio_elapsed / 60, props.cardio_elapsed % 60)
                            } else if cardio_done {
                                "Fatto".to_string()
                            } else {
                                "–".to_string()
                            }
                        }
                    } else {
                        { format!("{} / {}", completed_count, n) }
                    }
                </span>
                if video_embed.is_some() {
                    <button class="video-icon-btn" title="Guarda video esercizio"
                        onpointerdown={Callback::from(|e: PointerEvent| e.stop_propagation())}
                        onpointerup={Callback::from(|e: PointerEvent| e.stop_propagation())}
                        onclick={{
                            let vo = video_open.clone();
                            Callback::from(move |e: MouseEvent| {
                                e.stop_propagation();
                                vo.set(true);
                            })
                        }}>{ icon_play() }</button>
                }
                if video_embed.is_none() && !exercise.images.is_empty() {
                    <button class="video-icon-btn" title="Foto dimostrative esercizio"
                        onpointerdown={Callback::from(|e: PointerEvent| e.stop_propagation())}
                        onpointerup={Callback::from(|e: PointerEvent| e.stop_propagation())}
                        onclick={{
                            let po = photo_open.clone();
                            let pi = photo_idx.clone();
                            Callback::from(move |e: MouseEvent| {
                                e.stop_propagation();
                                pi.set(0);
                                po.set(true);
                            })
                        }}>{ icon_photo() }</button>
                }
                if !is_cardio && !is_timed {
                    <button class="chart-icon-btn" title="Grafico avanzamento peso"
                        onpointerdown={Callback::from(|e: PointerEvent| e.stop_propagation())}
                        onpointerup={Callback::from(|e: PointerEvent| e.stop_propagation())}
                        onclick={{
                            let co = chart_open.clone();
                            Callback::from(move |e: MouseEvent| {
                                e.stop_propagation();
                                co.set(!*co);
                            })
                        }}>{ icon_chart() }</button>
                }
            </div>

            // ── Expandable content ────────────────────────────────────────
            if *expanded {
                if is_cardio {
                    // ── Cardio UI ──────────────────────────────────────────
                    <div class="sheet-content" key={exercise_id.clone()}>
                        <ProgressBar
                            n={exercise.serie}
                            dot_done={dot_done.clone()}
                            dot_reps_hint={vec![]}
                            active={0}
                            just_saved={*just_saved}
                            on_select={Callback::from(|_: usize| {})}
                        />
                        <div class="cardio-stopwatch">
                            <div class="cardio-timer-wrap">
                                <span class="cardio-timer-label">{"timer"}</span>
                                <div class="cardio-time">
                                    { format!("{:02}:{:02}", props.cardio_elapsed / 60, props.cardio_elapsed % 60) }
                                </div>
                            </div>
                            <div class="cardio-target">
                                { format!("Obiettivo: {}", exercise.reps) }
                            </div>
                        </div>
                        if let Some(note) = &exercise.note {
                            <p class="exercise-note">{ note.clone() }</p>
                        }
                    </div>
                    if !props.history_mode {
                        <div class={if props.righthanded { "sheet-actions" } else { "sheet-actions sheet-actions--lh" }}>
                            if !props.righthanded {
                                <button class="primary-button" onclick={{
                                    let cb = props.on_cardio_toggle.clone();
                                    Callback::from(move |_: MouseEvent| cb.emit(()))
                                }}>
                                    { if props.cardio_running { "Pausa" }
                                      else if props.cardio_elapsed > 0 { "Riprendi" }
                                      else { "Avvia" } }
                                </button>
                            }
                            <button class="secondary-button" onclick={{
                                let cb = props.on_cardio_stop.clone();
                                Callback::from(move |_: MouseEvent| cb.emit(()))
                            }}
                                disabled={props.cardio_elapsed == 0 && !props.cardio_running}>
                                { if cardio_done { "Aggiorna" } else { "Registra" } }
                            </button>
                            if props.righthanded {
                                <button class="primary-button" onclick={{
                                    let cb = props.on_cardio_toggle.clone();
                                    Callback::from(move |_: MouseEvent| cb.emit(()))
                                }}>
                                    { if props.cardio_running { "Pausa" }
                                      else if props.cardio_elapsed > 0 { "Riprendi" }
                                      else { "Avvia" } }
                                </button>
                            }
                        </div>
                    }
                } else if is_timed {
                    // ── Timed exercise UI (countdown) ──────────────────────
                    <div class="sheet-content" key={exercise_id.clone()}>
                        <ProgressBar
                            n={exercise.serie}
                            dot_done={dot_done.clone()}
                            dot_reps_hint={vec![]}
                            active={clamped}
                            just_saved={*just_saved}
                            on_select={{
                                let asc = active_set.clone();
                                Callback::from(move |idx: usize| asc.set(idx))
                            }}
                        />
                        <div class="cardio-stopwatch">
                            <div class="cardio-timer-wrap">
                                <span class="cardio-timer-label">{"isometrico"}</span>
                                <div class="cardio-time">
                                    { format!("{:02}:{:02}",
                                        props.timed_timer.left / 60,
                                        props.timed_timer.left % 60)
                                    }
                                </div>
                            </div>
                            if let Some(dur) = exercise.durata {
                                <div class="cardio-target">
                                    { if dur >= 60 {
                                        format!("Obiettivo: {}'{:02}\"", dur / 60, dur % 60)
                                    } else {
                                        format!("Obiettivo: {}\"", dur)
                                    } }
                                </div>
                            }
                        </div>
                        if let Some(note) = &exercise.note {
                            <p class="exercise-note">{ note.clone() }</p>
                        }
                    </div>
                    if !props.history_mode {
                        <div class={if props.righthanded { "sheet-actions" } else { "sheet-actions sheet-actions--lh" }}>
                            if !props.righthanded {
                                <button class="primary-button" onclick={{
                                    let cb = props.on_timed_toggle.clone();
                                    Callback::from(move |_: MouseEvent| cb.emit(()))
                                }}>
                                    { if props.timed_timer.running { "Pausa" }
                                      else if props.timed_timer.left > 0 { "Riprendi" }
                                      else { "Avvia" } }
                                </button>
                            }
                            <button class="secondary-button" onclick={on_register.clone()} disabled={completed}>
                                { if completed { "Completata" } else { "Registra" } }
                            </button>
                            if props.righthanded {
                                <button class="primary-button" onclick={{
                                    let cb = props.on_timed_toggle.clone();
                                    Callback::from(move |_: MouseEvent| cb.emit(()))
                                }}>
                                    { if props.timed_timer.running { "Pausa" }
                                      else if props.timed_timer.left > 0 { "Riprendi" }
                                      else { "Avvia" } }
                                </button>
                            }
                        </div>
                    }
                } else {
                    // ── Weight / reps UI ───────────────────────────────────
                    <div class="sheet-content" key={exercise_id.clone()}>
                        if *chart_open {
                            <div class="weight-chart-section">
                                { render_weight_chart(&chart_points) }
                            </div>
                        }

                        <ProgressBar
                            n={exercise.serie}
                            dot_done={dot_done.clone()}
                            dot_reps_hint={dot_reps_hint.clone()}
                            active={clamped}
                            just_saved={*just_saved}
                            on_select={{
                                let asc = active_set.clone();
                                Callback::from(move |idx: usize| asc.set(idx))
                            }}
                        />

                        if let Some(rec) = exercise.recupero {
                            <div class="recovery-badge">
                                { if rec >= 60 {
                                    format!("Recupero: {}'{:02}\"", rec / 60, rec % 60)
                                } else {
                                    format!("Recupero: {}\"", rec)
                                } }
                            </div>
                        }

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
                                // ── Peso: input above, step buttons below ────────
                                <div class="input-field">
                                    <span class="input-label">{"Peso (kg)"}</span>
                                    <input class="weight-val-input" value={weight_display}
                                        inputmode="decimal"
                                        placeholder={if weight_hint.is_empty() { "0".to_string() } else { format!("{} (ultima)", weight_hint) }}
                                        oninput={{
                                            let cb = props.on_weight_change.clone();
                                            let eid = exercise_id.clone();
                                            Callback::from(move |e: InputEvent| {
                                                if let Some(i) = e.target().and_then(|t| t.dyn_into::<HtmlInputElement>().ok()) {
                                                    cb.emit((eid.clone(), clamped, i.value()));
                                                }
                                            })
                                        }}
                                    />
                                    <div class="step-row">
                                        <button class="step-btn" onclick={on_weight_minus}>{"−"}</button>
                                        <button class="step-selector" onclick={on_cycle_step}
                                                title="Tocca per cambiare passo">
                                            { format!("±{} kg", fmt_weight(step)) }
                                        </button>
                                        <button class="step-btn" onclick={on_weight_plus}>{"+"}</button>
                                    </div>
                                </div>
                                // ── Reps: buttons beside the input ───────────────
                                <div class="input-field">
                                    <span class="input-label">{"Reps"}</span>
                                    <div class="reps-row">
                                        <button class="step-btn" onclick={on_reps_minus}>{"−"}</button>
                                        <input class="reps-val-input" value={reps_display}
                                            inputmode="numeric"
                                            placeholder={if reps_hint.is_empty() { exercise.reps.clone() } else { format!("{} (ultima)", reps_hint) }}
                                            oninput={{
                                                let cb = props.on_reps_change.clone();
                                                let eid = exercise_id.clone();
                                                Callback::from(move |e: InputEvent| {
                                                    if let Some(i) = e.target().and_then(|t| t.dyn_into::<HtmlInputElement>().ok()) {
                                                        cb.emit((eid.clone(), clamped, i.value()));
                                                    }
                                                })
                                            }}
                                        />
                                        <button class="step-btn" onclick={on_reps_plus}>{"+"}</button>
                                    </div>
                                </div>
                            </div>
                        </div>

                        if let Some(note) = &exercise.note {
                            <p class="exercise-note">{ note.clone() }</p>
                        }
                    </div>
                    // ── Action footer — outside scrollable area, always visible ──
                    <div class={if props.righthanded { "sheet-actions" } else { "sheet-actions sheet-actions--lh" }}>
                        if props.righthanded {
                            // Right-handed: [Registra serie secondary] [Avvia recupero primary] — timer closest to thumb
                        } else {
                            // Left-handed: [Avvia recupero primary] first — timer closest to thumb
                            if !props.history_mode
                                && (!completed || props.timer.running || props.timer.left > 0)
                            {
                                <button class="primary-button" onclick={{
                                    let cb = props.on_start_timer.clone();
                                    Callback::from(move |_: MouseEvent| cb.emit(()))
                                }}>
                                    { if props.timer.running { "Pausa" }
                                      else if props.timer.left > 0 { "Riprendi recupero" }
                                      else { "Avvia recupero" } }
                                </button>
                            }
                        }
                        <button class="secondary-button" onclick={on_register.clone()}
                            disabled={props.timer.running || props.timer.left > 0}>
                            { if completed { "Aggiorna serie" } else { "Registra serie" } }
                        </button>
                        if props.righthanded {
                            // Right-handed: [Avvia recupero primary] last — timer closest to thumb
                            if !props.history_mode
                                && (!completed || props.timer.running || props.timer.left > 0)
                            {
                                <button class="primary-button" onclick={{
                                    let cb = props.on_start_timer.clone();
                                    Callback::from(move |_: MouseEvent| cb.emit(()))
                                }}>
                                    { if props.timer.running { "Pausa" }
                                      else if props.timer.left > 0 { "Riprendi recupero" }
                                      else { "Avvia recupero" } }
                                </button>
                            }
                        }
                    </div>
                }
            }
        </div>

        // ── Video overlay (position:fixed, renders over everything) ───────
        if *video_open {
            if let Some(embed_url) = video_embed {
                <div class="video-overlay"
                    tabindex="0"
                    role="dialog"
                    aria-modal="true"
                    aria-label="Video esercizio"
                    onclick={close_video.clone()}
                    onkeydown={close_video_on_escape}>
                    <div class="video-modal"
                        onclick={Callback::from(|e: MouseEvent| e.stop_propagation())}>
                        <button class="menu-close-btn video-close-btn"
                            aria-label="Chiudi video esercizio"
                            autofocus={true}
                            onclick={close_video.clone()}>
                            {"✕"}
                        </button>
                        <div class="video-iframe-wrapper">
                            <iframe
                                title={format!("Video esercizio {}", exercise.display_name())}
                                src={embed_url}
                                allow="autoplay; encrypted-media; fullscreen"
                                allowfullscreen={true}
                                style="position:absolute;inset:0;width:100%;height:100%;border:0;border-radius:14px;"
                            />
                        </div>
                    </div>
                </div>
            }
        }

        // ── Photo carousel overlay (position:fixed, renders over everything) ──
        if *photo_open {
            if let Some(src) = current_photo_url {
                <div class="video-overlay"
                    tabindex="0"
                    role="dialog"
                    aria-modal="true"
                    aria-label="Foto dimostrative esercizio"
                    onclick={close_photo.clone()}
                    onkeydown={close_photo_on_escape}>
                    <div class="video-modal photo-modal"
                        onclick={Callback::from(|e: MouseEvent| e.stop_propagation())}>
                        <button class="menu-close-btn video-close-btn"
                            aria-label="Chiudi foto dimostrative"
                            autofocus={true}
                            onclick={close_photo.clone()}>
                            {"✕"}
                        </button>
                        <div class="photo-frame">
                            <img src={src} alt={format!("Dimostrazione esercizio {}", exercise.display_name())} class="photo-img" />
                            if n_photos > 1 {
                                <button class="photo-nav photo-nav--prev" title="Precedente"
                                    aria-label="Foto precedente"
                                    onclick={{
                                        let pi = photo_idx.clone();
                                        Callback::from(move |_: MouseEvent| {
                                            pi.set((*pi + n_photos - 1) % n_photos);
                                        })
                                    }}>{"‹"}</button>
                                <button class="photo-nav photo-nav--next" title="Successiva"
                                    aria-label="Foto successiva"
                                    onclick={{
                                        let pi = photo_idx.clone();
                                        Callback::from(move |_: MouseEvent| {
                                            pi.set((*pi + 1) % n_photos);
                                        })
                                    }}>{"›"}</button>
                                <span class="photo-counter">{ format!("{} / {}", photo_clamped + 1, n_photos) }</span>
                            }
                        </div>
                    </div>
                </div>
            }
        }
        </>
    }
}
