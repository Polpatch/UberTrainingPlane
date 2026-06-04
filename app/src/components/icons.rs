use yew::prelude::*;

// Each icon is a complete SVG string parsed via from_html_unchecked so that
// the browser receives a proper <svg> root — which triggers SVG namespace for
// all child elements (paths, circles, etc.). Injecting only the inner path
// into a Yew-rendered <svg> does NOT work because from_html_unchecked parses
// in HTML context, where <path> is an unknown element without SVG namespace.

const ATTRS: &str = r#"viewBox="0 0 24 24" width="1em" height="1em" aria-hidden="true" style="display:block;flex-shrink:0" xmlns="http://www.w3.org/2000/svg""#;

fn svg(inner: &str) -> Html {
    Html::from_html_unchecked(
        format!("<svg {ATTRS}>{inner}</svg>").into()
    )
}

pub fn icon_play() -> Html {
    svg(r#"<path fill="currentColor" fill-rule="evenodd" clip-rule="evenodd" d="M4.5 5.653c0-1.426 1.529-2.33 2.779-1.643l11.54 6.348c1.295.712 1.295 2.573 0 3.285L7.28 19.991c-1.25.687-2.779-.217-2.779-1.643V5.653z"/>"#)
}

pub fn icon_pause() -> Html {
    svg(r#"<path fill="currentColor" fill-rule="evenodd" clip-rule="evenodd" d="M6.75 5.25a.75.75 0 01.75-.75H9a.75.75 0 01.75.75v13.5a.75.75 0 01-.75.75H7.5a.75.75 0 01-.75-.75V5.25zm7.5 0A.75.75 0 0115 4.5h1.5a.75.75 0 01.75.75v13.5a.75.75 0 01-.75.75H15a.75.75 0 01-.75-.75V5.25z"/>"#)
}

pub fn icon_skip() -> Html {
    svg(r#"<path fill="currentColor" d="M5.055 7.06C3.805 6.347 2.25 7.25 2.25 8.69v8.122c0 1.44 1.555 2.342 2.805 1.628L12 14.471v2.34c0 1.44 1.555 2.342 2.805 1.628l7.108-4.061c1.26-.72 1.26-2.536 0-3.256L14.805 7.06C13.555 6.347 12 7.25 12 8.69v2.34L5.055 7.06z"/>"#)
}

pub fn icon_x() -> Html {
    svg(r#"<path stroke="currentColor" stroke-width="2" stroke-linecap="round" fill="none" d="M6 6l12 12M18 6L6 18"/>"#)
}

pub fn icon_clock() -> Html {
    svg(r#"<path stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" fill="none" d="M12 6v6l3.5 3.5M21 12a9 9 0 11-18 0 9 9 0 0118 0z"/>"#)
}

pub fn icon_download() -> Html {
    svg(r#"<path stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" fill="none" d="M12 3v13m0 0l-4-4m4 4l4-4M3 21h18"/>"#)
}

pub fn icon_upload() -> Html {
    svg(r#"<path stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" fill="none" d="M12 21V8m0 0l-4 4m4-4l4 4M3 3h18"/>"#)
}

pub fn icon_document() -> Html {
    svg(r#"<path stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" fill="none" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"/>"#)
}

pub fn icon_code() -> Html {
    svg(r#"<path stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" fill="none" d="M17 8l4 4-4 4M7 8l-4 4 4 4M14 4l-4 16"/>"#)
}

pub fn icon_chart() -> Html {
    svg(r#"<path stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" fill="none" d="M3 3v18h18M7 16v-4m4 4V8m4 8V4"/>"#)
}
