# Export and review

Render captures slide content at logical size with no controls, then writes a
labeled contact sheet. Open both the overview and relevant full-size PNGs before
claiming visual review. Inspect exported PDF when a PDF viewer is available.

HTML export is static and assembled, with relative asset links. It is verified via
HTTP with external network blocked and navigation exercised. Serve dist/html with
an ordinary static server (e.g. python3 -m http.server 8080 --directory dist/html).
No Rust/Node runtime is needed to view it. file:// is not supported.

PDF has one page per slide in manifest order, explicit page dimensions, backgrounds,
zero browser margins/headers, and a parsed page-count check. Browser-rendered text
remains selectable where possible. Interactive state is set by export hooks.

OS/system-font differences prevent universal pixel identity. Unregistered async
work, arbitrary timers and custom print CSS can break reproducibility. Report the
actual tool result, not an assumption that every export succeeded. HTML includes
embedded speaker notes. Generated exports exclude development dependencies.
