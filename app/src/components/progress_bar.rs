use std::cell::RefCell;
use std::rc::Rc;
use web_sys::TouchEvent;
use yew::prelude::*;

const MAX_VISIBLE: usize = 5;
const STEP_PX:    f64 = 54.0; // DOT (34px) + GAP (20px)
const HALF_DOT:   f64 = 17.0; // DOT / 2

#[derive(Properties, PartialEq)]
pub struct ProgressBarProps {
    /// Number of sets for this exercise.
    pub n: u32,
    /// Completion state per set index (true = already done).
    pub dot_done: Vec<bool>,
    /// Currently active set index (0-based, clamped).
    pub active: usize,
    /// Called with the new index when the user taps a dot.
    pub on_select: Callback<usize>,
}

#[function_component(ProgressBar)]
pub fn progress_bar(props: &ProgressBarProps) -> Html {
    let n           = props.n as usize;
    let base_offset = use_state(|| HALF_DOT);
    let is_dragging = use_state(|| false);
    let drag_start: Rc<RefCell<Option<(f64, f64)>>> = use_mut_ref(|| None);

    let max_offset    = n.saturating_sub(1) as f64 * STEP_PX + HALF_DOT;
    let needs_carousel = n > MAX_VISIBLE;

    // Re-centre the active dot whenever it changes (e.g. auto-advance from parent).
    {
        let bo  = base_offset.clone();
        let max = max_offset;
        use_effect_with_deps(
            move |idx: &usize| {
                let target = *idx as f64 * STEP_PX + HALF_DOT;
                bo.set(target.max(HALF_DOT).min(max));
                || ()
            },
            props.active,
        );
    }

    // ── Snap: round to nearest slot ────────────────────────────────────────
    let snap = {
        let bo  = base_offset.clone();
        let ds  = drag_start.clone();
        let isd = is_dragging.clone();
        let max = max_offset;
        move || {
            let raw     = *bo;
            let idx     = ((raw - HALF_DOT) / STEP_PX).round();
            let snapped = idx * STEP_PX + HALF_DOT;
            bo.set(snapped.max(HALF_DOT).min(max));
            *ds.borrow_mut() = None;
            isd.set(false);
        }
    };

    // ── Mouse drag ─────────────────────────────────────────────────────────
    let on_mouse_down = {
        let bo  = base_offset.clone();
        let ds  = drag_start.clone();
        let isd = is_dragging.clone();
        Callback::from(move |e: MouseEvent| {
            *ds.borrow_mut() = Some((e.client_x() as f64, *bo));
            isd.set(true);
        })
    };

    let on_mouse_move = {
        let bo   = base_offset.clone();
        let ds   = drag_start.clone();
        let snap = snap.clone();
        let max  = max_offset;
        Callback::from(move |e: MouseEvent| {
            if e.buttons() == 0 {
                if ds.borrow().is_some() { snap(); }
                return;
            }
            if let Some((sx, so)) = *ds.borrow() {
                let new_off = (so + sx - e.client_x() as f64).max(HALF_DOT).min(max);
                bo.set(new_off);
            }
        })
    };

    let on_mouse_up = {
        let snap = snap.clone();
        Callback::from(move |_: MouseEvent| snap())
    };

    // ── Touch drag ─────────────────────────────────────────────────────────
    let on_touch_start = {
        let bo  = base_offset.clone();
        let ds  = drag_start.clone();
        let isd = is_dragging.clone();
        Callback::from(move |e: TouchEvent| {
            if let Some(t) = e.touches().get(0) {
                *ds.borrow_mut() = Some((t.client_x() as f64, *bo));
                isd.set(true);
            }
        })
    };

    let on_touch_move = {
        let bo  = base_offset.clone();
        let ds  = drag_start.clone();
        let max = max_offset;
        Callback::from(move |e: TouchEvent| {
            if let Some(t) = e.touches().get(0) {
                if let Some((sx, so)) = *ds.borrow() {
                    let new_off = (so + sx - t.client_x() as f64).max(HALF_DOT).min(max);
                    bo.set(new_off);
                }
            }
        })
    };

    let on_touch_end = {
        let snap = snap.clone();
        Callback::from(move |_: TouchEvent| snap())
    };

    // ── Dot factory ────────────────────────────────────────────────────────
    let make_dot = {
        let dot_done    = props.dot_done.clone();
        let active      = props.active;
        let on_select   = props.on_select.clone();
        move |idx: usize| -> Html {
            let is_done   = dot_done.get(idx).copied().unwrap_or(false);
            let is_active = active == idx;
            let on_sel    = on_select.clone();
            html! {
                <button
                    class={classes!(
                        "series-dot",
                        if is_done   { Some("completed") } else { None },
                        if is_active { Some("active")    } else { None }
                    )}
                    onclick={Callback::from(move |e: MouseEvent| {
                        e.stop_propagation();
                        on_sel.emit(idx);
                    })}
                >{ (idx + 1).to_string() }</button>
            }
        }
    };

    // ── Rendering ──────────────────────────────────────────────────────────
    if n == 1 {
        let is_done = props.dot_done.first().copied().unwrap_or(false);
        return html! {
            <div class="series-progress">
                <div class={classes!(
                    "series-capsule",
                    if is_done { Some("completed") } else { Some("active") }
                )}></div>
            </div>
        };
    }

    if !needs_carousel {
        // Simple row — lines fill available width with flex:1
        let items: Vec<Html> = (0..n).flat_map(|idx| {
            let dot = make_dot(idx);
            if idx < n - 1 {
                vec![dot, html! { <div class="series-line"></div> }]
            } else {
                vec![dot]
            }
        }).collect();
        return html! {
            <div class="series-progress">
                { for items.into_iter() }
            </div>
        };
    }

    // Carousel — single translating track, clipped by overflow:hidden container
    let translate = format!(
        "transform: translateX(-{}px); transition: {}",
        *base_offset,
        if *is_dragging { "none" } else { "transform 0.25s ease" }
    );
    let items: Vec<Html> = (0..n).flat_map(|idx| {
        let dot = make_dot(idx);
        if idx < n - 1 {
            vec![dot, html! { <div class="series-line series-line--c"></div> }]
        } else {
            vec![dot]
        }
    }).collect();

    html! {
        <div
            class="series-progress"
            style="padding:0"  // override the 12% padding — carousel uses its own container
        >
            <div
                class="carousel-outer"
                style="margin:0;width:100%"
                onmousedown={on_mouse_down}
                onmousemove={on_mouse_move}
                onmouseup={on_mouse_up.clone()}
                onmouseleave={on_mouse_up}
                ontouchstart={on_touch_start}
                ontouchmove={on_touch_move}
                ontouchend={on_touch_end}
            >
                <div class="carousel-track" style={translate}>
                    { for items.into_iter() }
                </div>
            </div>
        </div>
    }
}
