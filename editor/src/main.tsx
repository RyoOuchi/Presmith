import { useEffect, useMemo, useRef, useState } from "react";
import { createRoot } from "react-dom/client";
import { apply, type Project, type Operation, type Selection } from "./types";
import "./style.css";
const credential =
  new URLSearchParams(location.hash.slice(1)).get("session") ||
  sessionStorage.getItem("decksmith-session") ||
  "";
if (credential) sessionStorage.setItem("decksmith-session", credential);
history.replaceState(null, "", location.pathname);
async function api(path: string, body?: object) {
  const r = await fetch(`/api/${path}`, {
    method: body ? "POST" : "GET",
    headers: {
      "x-decksmith-session": credential,
      ...(body ? { "Content-Type": "application/json" } : {}),
    },
    body: body ? JSON.stringify(body) : undefined,
  });
  const result = await r.json();
  if (!r.ok) throw new Error(result.error || `Request failed (${r.status})`);
  return result;
}
function Field({
  label,
  value,
  onCommit,
  disabled,
  reason,
  multiline = false,
  options,
  unit,
  mark = false,
}: {
  label: string;
  value: string;
  onCommit: (s: string) => void;
  disabled?: boolean;
  reason?: string;
  multiline?: boolean;
  options?: string[];
  unit?: string;
  mark?: boolean;
}) {
  const [draft, setDraft] = useState(value);
  const input = useRef<
    HTMLInputElement | HTMLTextAreaElement | HTMLSelectElement | null
  >(null);
  const composing = useRef(false),
    changed = useRef(false);
  useEffect(() => {
    if (!changed.current) setDraft(value);
  }, [value]);
  const announce = (dirty: boolean) =>
    window.dispatchEvent(
      new CustomEvent("editor-draft", { detail: { label, dirty } }),
    );
  useEffect(
    () => () => {
      announce(false);
    },
    [],
  );
  const commit = () => {
    if (composing.current) return;
    if (changed.current && draft !== value) onCommit(draft);
    changed.current = false;
    announce(false);
  };
  const shared = {
    value: draft,
    disabled,
    "aria-label": label,
    title: reason,
    onChange: (
      e: React.ChangeEvent<
        HTMLInputElement | HTMLTextAreaElement | HTMLSelectElement
      >,
    ) => {
      changed.current = true;
      setDraft(e.target.value);
      announce(e.target.value !== value);
    },
    onBlur: commit,
    onCompositionStart: () => (composing.current = true),
    onCompositionEnd: (
      e: React.CompositionEvent<
        HTMLInputElement | HTMLTextAreaElement | HTMLSelectElement
      >,
    ) => {
      composing.current = false;
      if (document.activeElement !== input.current) {
        if (e.currentTarget.value !== value) onCommit(e.currentTarget.value);
        announce(false);
      }
    },
    onKeyDown: (e: React.KeyboardEvent) => {
      if (e.key === "Enter" && !multiline && !composing.current) {
        e.preventDefault();
        input.current?.blur();
      }
    },
  };
  return (
    <label className={`field ${disabled ? "disabled" : ""}`} title={reason}>
      <span>
        {label}
        {unit && <small>{unit}</small>}
        {mark && <i title="GUI override active">●</i>}
      </span>
      {multiline ? (
        <textarea
          ref={(n) => {
            input.current = n;
          }}
          {...shared}
          rows={4}
        />
      ) : options ? (
        <select
          ref={(n) => {
            input.current = n;
          }}
          {...shared}
          onChange={(e) => {
            setDraft(e.target.value);
            onCommit(e.target.value);
          }}
        >
          {!options.includes(draft) && (
            <option value={draft}>{draft || "Authored"}</option>
          )}
          {options.map((v) => (
            <option key={v}>{v}</option>
          ))}
        </select>
      ) : (
        <input
          ref={(n) => {
            input.current = n;
          }}
          {...shared}
        />
      )}
    </label>
  );
}
const names: Record<string, string> = {
  "font-family": "Font family",
  "font-size": "Font size",
  "font-weight": "Weight",
  "font-style": "Emphasis",
  "text-decoration-line": "Decoration",
  "text-align": "Text alignment",
  color: "Text color",
  "background-color": "Background",
  width: "Width",
  height: "Height",
  padding: "Padding",
  gap: "Gap",
  "row-gap": "Row gap",
  "column-gap": "Column gap",
  "border-color": "Border color",
  "border-width": "Border width",
  "border-style": "Border style",
  "border-radius": "Corner radius",
  "object-fit": "Object fit",
  left: "Left",
  top: "Top",
  "align-items": "Align items",
  "justify-content": "Justify content",
  "flex-direction": "Direction",
  "align-self": "Align self",
  order: "Layout order",
};
const options: Record<string, string[]> = {
  "font-weight": [
    "100",
    "200",
    "300",
    "400",
    "500",
    "600",
    "650",
    "700",
    "800",
    "900",
  ],
  "font-style": ["normal", "italic"],
  "text-decoration-line": ["none", "underline", "line-through"],
  "text-align": ["left", "center", "right", "justify"],
  "object-fit": ["contain", "cover", "fill", "none", "scale-down"],
  "border-style": ["none", "solid", "dashed", "dotted", "double"],
  "align-items": ["stretch", "start", "end", "center", "baseline"],
  "align-self": ["auto", "stretch", "start", "end", "center"],
  "justify-content": [
    "start",
    "end",
    "center",
    "space-between",
    "space-around",
    "space-evenly",
  ],
  "flex-direction": ["row", "column", "row-reverse", "column-reverse"],
};
const pixels = new Set([
  "font-size",
  "width",
  "height",
  "padding",
  "gap",
  "row-gap",
  "column-gap",
  "border-width",
  "border-radius",
  "left",
  "top",
]);
function hex(value: string) {
  const c = value.match(/^rgba?\((\d+), (\d+), (\d+)(?:, ([\d.]+))?\)$/);
  if (!c) return value;
  if (c[4] === "0") return "transparent";
  return (
    "#" +
    c
      .slice(1, 4)
      .map((v) => Number(v).toString(16).padStart(2, "0"))
      .join("")
  );
}
interface Command {
  label: string;
  ops: Operation[];
}
function App() {
  const [draftDirty, setDraftDirty] = useState(false);
  const draftFields = useRef(new Set<string>());
  const [base, setBase] = useState<Project | null>(null),
    [history, setHistory] = useState<Command[]>([]),
    [cursor, setCursor] = useState(0),
    [saved, setSaved] = useState("[]"),
    [revision, setRevision] = useState("");
  const [slide, setSlide] = useState(""),
    [selected, setSelected] = useState<string | null>(null),
    [selection, setSelection] = useState<Selection | null>(null),
    [mode, setMode] = useState("edit"),
    [zoom, setZoom] = useState<number | "fit">("fit"),
    [fit, setFit] = useState(0.5),
    [status, setStatus] = useState("Saved"),
    [error, setError] = useState(""),
    [conflict, setConflict] = useState(false),
    [effects, setEffects] = useState<string[]>([]),
    [busy, setBusy] = useState(false),
    [ready, setReady] = useState(0),
    [typing, setTyping] = useState(false),
    [exports, setExports] = useState<string[]>([]);
  const validation = useRef<{ id: number; resolve: (v: any) => void } | null>(
    null,
  );
  const iframe = useRef<HTMLIFrameElement>(null),
    canvas = useRef<HTMLDivElement>(null),
    thumbs = useRef(new Map<string, HTMLIFrameElement>()),
    dragSlide = useRef<string | null>(null),
    latest = useRef<any>(null);
  const ops = useMemo(
    () => history.slice(0, cursor).flatMap((c) => c.ops),
    [history, cursor],
  );
  const model = useMemo(() => (base ? apply(base, ops) : null), [base, ops]);
  const dirty = JSON.stringify(ops) !== saved || draftDirty;
  const node = model?.elements[slide]?.find((n) => n.key === selected);
  const active = model?.manifest.slides.find((s) => s.id === slide);
  const scale = zoom === "fit" ? fit : zoom;
  function add(label: string, ops: Operation[]) {
    const v = latest.current;
    setHistory([...v.history.slice(0, v.cursor), { label, ops }]);
    setCursor(v.cursor + 1);
    setError("");
    setStatus("Unsaved changes");
  }
  function undo() {
    const v = latest.current;
    if (v.busy || v.typing) return;
    setCursor(Math.max(0, v.cursor - 1));
    setError("");
  }
  function redo() {
    const v = latest.current;
    if (v.busy || v.typing) return;
    setCursor(Math.min(v.history.length, v.cursor + 1));
    setError("");
  }
  function commitText(s: string, e: string, html: string) {
    const dom = new DOMParser().parseFromString(
      `<body>${html}</body>`,
      "text/html",
    );
    const valid = [...dom.body.querySelectorAll("*")].every(
      (n) =>
        ["STRONG", "EM", "B", "I", "U", "S", "CODE", "BR"].includes(
          n.tagName,
        ) && !n.attributes.length,
    );
    if (!valid) {
      setError(
        "Text supports bold, emphasis, underline, strike, code and line breaks only. Unsupported markup was not committed.",
      );
      setReady((v) => v + 1);
      return;
    }
    add("Edit text", [{ type: "text", slide: s, element: e, html }]);
  }
  async function load(discard = false) {
    setBusy(true);
    setStatus(discard ? "Reloading…" : "Opening…");
    try {
      const p = await api(
        discard ? "reload" : "project",
        discard ? { discard: true } : undefined,
      );
      setBase(p);
      setRevision(p.revision);
      setSaved("[]");
      setHistory([]);
      setCursor(0);
      setSlide(p.manifest.slides[0].id);
      setSelected(null);
      setSelection(null);
      setError("");
      setConflict(false);
      setEffects([]);
      setStatus("Saved");
    } catch (e) {
      setError(String(e));
    } finally {
      setBusy(false);
    }
  }
  async function save() {
    (document.activeElement as HTMLElement)?.blur();
    await new Promise((r) => setTimeout(r, 30));
    const v = latest.current;
    if (!v.base || v.busy || v.typing) return;
    if (v.effects.length) {
      setError(v.effects.join(" "));
      return;
    }
    setBusy(true);
    setStatus("Saving…");
    try {
      const check = await new Promise<{ effects?: string[]; error?: string }>(
        (resolve) => {
          const id = Date.now();
          validation.current = { id, resolve };
          iframe.current?.contentWindow?.postMessage(
            { decksmith: true, type: "validate", id },
            "*",
          );
          setTimeout(() => {
            if (validation.current?.id === id) {
              validation.current = null;
              resolve({
                error:
                  "Canvas validation timed out. Pending edits are retained. Retry Save after the preview is ready.",
              });
            }
          }, 30000);
        },
      );
      if (check.error) throw new Error(check.error);
      if (check.effects?.length) {
        setEffects(check.effects);
        throw new Error(check.effects.join(" "));
      }
      const r = await api("save", {
        baseline: v.base.revision,
        revision: v.revision,
        operations: v.ops,
      });
      setRevision(r.revision);
      setSaved(JSON.stringify(v.ops));
      setConflict(false);
      setStatus("Saved");
      setError("");
    } catch (e) {
      const message = String(e);
      setError(message);
      if (message.includes("CONFLICT")) setConflict(true);
      setStatus(message.includes("CONFLICT") ? "Conflict" : "Save failed");
    } finally {
      setBusy(false);
    }
  }
  function reorder(id: string, to: number) {
    if (!model) return;
    const order = model.manifest.slides.map((s) => s.id),
      from = order.indexOf(id);
    if (to < 0 || to >= order.length || from === to) return;
    order.splice(from, 1);
    order.splice(to, 0, id);
    add("Reorder slides", [{ type: "reorder", order }]);
  }
  async function exportDeck(format: string) {
    setBusy(true);
    setStatus("Exporting…");
    setError("");
    try {
      const r = await api("export", { format, revision });
      setExports(r.artifacts.map((a: { path: string }) => a.path));
      setStatus("Export complete");
    } catch (e) {
      setError(String(e));
    } finally {
      setBusy(false);
    }
  }
  function downloadPending() {
    const url = URL.createObjectURL(
      new Blob([JSON.stringify({ revision, operations: ops }, null, 2)], {
        type: "application/json",
      }),
    );
    const a = document.createElement("a");
    a.href = url;
    a.download = "presmith-pending-edits.json";
    a.click();
    URL.revokeObjectURL(url);
  }
  latest.current = {
    base,
    model,
    history,
    cursor,
    ops,
    revision,
    dirty,
    effects,
    busy,
    typing,
    save,
    undo,
    redo,
    add,
    commitText,
  };
  useEffect(() => {
    load();
    const listen = (e: Event) => {
      const d = (e as CustomEvent).detail;
      if (d.dirty) draftFields.current.add(d.label);
      else draftFields.current.delete(d.label);
      setDraftDirty(draftFields.current.size > 0);
    };
    window.addEventListener("editor-draft", listen);
    return () => window.removeEventListener("editor-draft", listen);
  }, []);
  useEffect(() => {
    if (!base) return;
    const ro = new ResizeObserver(() => {
      if (canvas.current)
        setFit(
          Math.min(
            (canvas.current.clientWidth - 64) / base.manifest.width,
            (canvas.current.clientHeight - 64) / base.manifest.height,
          ),
        );
    });
    ro.observe(canvas.current!);
    return () => ro.disconnect();
  }, [base]);
  useEffect(() => {
    const keydown = (e: KeyboardEvent) => {
      if (!(e.metaKey || e.ctrlKey)) return;
      if (e.key.toLowerCase() === "s") {
        e.preventDefault();
        latest.current.save();
      }
      if (
        e.key.toLowerCase() === "z" &&
        !(e.target as HTMLElement).closest("input,textarea,[contenteditable]")
      ) {
        e.preventDefault();
        latest.current[e.shiftKey ? "redo" : "undo"]();
      }
    };
    const before = (e: BeforeUnloadEvent) => {
      if (latest.current?.dirty || latest.current?.typing) {
        e.preventDefault();
        e.returnValue = "";
      }
    };
    window.addEventListener("keydown", keydown);
    window.addEventListener("beforeunload", before);
    return () => {
      window.removeEventListener("keydown", keydown);
      window.removeEventListener("beforeunload", before);
    };
  }, []);
  useEffect(() => {
    const timer = setInterval(async () => {
      if (!latest.current?.base || latest.current.busy) return;
      try {
        const s = await api("status");
        if (s.revision !== latest.current.revision) {
          setConflict(true);
          setStatus("Conflict");
        }
      } catch (e) {
        if (String(e).includes("CONFLICT")) setConflict(true);
        setError(`Source check failed. Pending edits are retained. ${e}`);
      }
    }, 4000);
    return () => clearInterval(timer);
  }, []);
  function sendFrame(
    frame: HTMLIFrameElement | null,
    id: string,
    mode: string,
  ) {
    if (!model || !frame) return;
    frame.contentWindow?.postMessage(
      {
        decksmith: true,
        type: "sync",
        slide: id,
        mode,
        scale: mode === "thumbnail" ? 1 : scale,
        key: mode === "thumbnail" ? null : selected,
        elements: model.elements[id],
        allElements: model.elements,
        overrides: model.overrides,
      },
      "*",
    );
  }
  useEffect(() => {
    sendFrame(iframe.current, slide, mode);
    for (const [id, frame] of thumbs.current) sendFrame(frame, id, "thumbnail");
  }, [model, slide, selected, mode, ready, scale]);
  useEffect(() => {
    const listener = (event: MessageEvent) => {
      if (!event.data?.decksmith) return;
      const d = event.data;
      if (event.source !== iframe.current?.contentWindow) {
        if (d.type === "ready") {
          for (const [id, f] of thumbs.current)
            if (f.contentWindow === event.source) sendFrame(f, id, "thumbnail");
        }
        return;
      }
      if (d.type === "ready") setReady((v) => v + 1);
      if (d.type === "selection") {
        setSelected(d.selection.key);
        setSelection(d.selection);
      }
      if (d.type === "readonly") {
        setSelected(null);
        setSelection(null);
        setError(d.message);
      }
      if (d.type === "effects") setEffects(d.effects);
      if (
        d.type === "validated" &&
        validation.current &&
        validation.current.id === d.id
      ) {
        validation.current.resolve(d);
        validation.current = null;
      }
      if (d.type === "editing") setTyping(d.active);
      if (d.type === "error") setError(d.message);
      if (d.type === "text")
        latest.current.commitText(d.slide, d.element, d.html);
      if (d.type === "geometry")
        latest.current.add(
          "Change geometry",
          Object.entries(d.values).map(([property, value]) => ({
            type: "style",
            slide: d.slide,
            element: d.element,
            property,
            value,
          })),
        );
      if (d.type === "undo" || d.type === "redo") latest.current[d.type]();
      if (d.type === "savehint")
        setError(
          "Use Save in the editor toolbar, or press ⌘/Ctrl S while the editor controls have focus.",
        );
      if (d.type === "slide") setSlide(d.slide);
    };
    window.addEventListener("message", listener);
    return () => window.removeEventListener("message", listener);
  }, [model, slide, selected, mode, scale]);
  function pick(id: string) {
    setSlide(id);
    setSelected(null);
    setSelection(null);
    setError("");
  }
  function prop(p: string) {
    if (!node || !selection) return null;
    const override = model!.overrides[slide]?.[node.key]?.[p];
    const disabled = !!selection.blocked[p] || !!selection.readOnly;
    let value = override ?? selection.computed[p] ?? "";
    if (p.includes("color")) value = hex(value);
    if (pixels.has(p)) value = value.replace(/px$/, "");
    return (
      <div className="property" key={p}>
        <Field
          label={names[p]}
          value={value}
          unit={pixels.has(p) ? "px" : undefined}
          disabled={disabled}
          reason={selection.blocked[p] || selection.readOnly}
          mark={override !== undefined}
          options={options[p]}
          onCommit={(v) =>
            add(`Set ${names[p]}`, [
              {
                type: "style",
                slide,
                element: node.key,
                property: p,
                value: pixels.has(p) ? `${v}px` : v,
              },
            ])
          }
        />
        <button
          className="reset"
          aria-label={`Reset ${names[p]}`}
          title="Remove GUI override and reveal authored style"
          disabled={override === undefined}
          onClick={() =>
            add(`Reset ${names[p]}`, [
              {
                type: "style",
                slide,
                element: node.key,
                property: p,
                value: null,
              },
            ])
          }
        >
          ↺
        </button>
      </div>
    );
  }
  if (!model)
    return (
      <main className="loading">
        <div className="brandmark">P</div>
        <h1>Presmith</h1>
        <p>{error || "Opening your source files…"}</p>
        <button onClick={() => load()}>Retry</button>
      </main>
    );
  return (
    <div className="app">
      <header className="toolbar">
        <div className="brandmark">P</div>
        <div className="document-title">
          <strong>{model.manifest.title}</strong>
          <span>Presmith editor</span>
        </div>
        <div className="history">
          <button
            aria-label="Undo"
            title="Undo · ⌘/Ctrl Z"
            disabled={!cursor || busy || typing}
            onClick={undo}
          >
            ↶
          </button>
          <button
            aria-label="Redo"
            title="Redo · ⇧⌘/Ctrl Z"
            disabled={cursor === history.length || busy || typing}
            onClick={redo}
          >
            ↷
          </button>
        </div>
        <output
          className={`status ${conflict ? "conflict" : dirty ? "dirty" : ""}`}
          aria-live="polite"
        >
          ●{" "}
          {busy
            ? status
            : conflict
              ? "Conflict"
              : error
                ? "Error"
                : typing
                  ? "Editing text…"
                  : dirty
                    ? "Unsaved changes"
                    : status === "Export complete"
                      ? status
                      : "Saved"}
        </output>
        <button
          className={mode === "preview" ? "active" : ""}
          disabled={busy}
          onClick={() => setMode(mode === "edit" ? "preview" : "edit")}
        >
          {mode === "edit" ? "▷ Preview" : "✎ Edit"}
        </button>
        <select
          aria-label="Export format"
          disabled={dirty || busy || conflict || typing}
          title={
            dirty
              ? "Save changes before exporting"
              : "Export saved source using Presmith"
          }
          value=""
          onChange={(e) => exportDeck(e.target.value)}
        >
          <option value="" disabled>
            Export ↗
          </option>
          {["html", "png", "pdf", "pptx"].map((f) => (
            <option key={f} value={f}>
              {f.toUpperCase()}
            </option>
          ))}
        </select>
        <button
          className="primary"
          onClick={save}
          title={
            typing
              ? "Click outside the canvas text to finish editing"
              : "Save source · ⌘/Ctrl S"
          }
          disabled={busy || typing || conflict || !!effects.length}
        >
          Save
        </button>
      </header>
      {(error || conflict || effects.length > 0 || exports.length > 0) && (
        <div className={`notice ${conflict ? "warning" : ""}`} role="status">
          {conflict ? (
            <>
              <strong>Source changed outside this editor.</strong> Your pending
              edits are retained. {error && <span>{error} </span>}
              <button onClick={downloadPending}>Download pending edits</button>
              <button
                onClick={() => {
                  if (
                    confirm(
                      "Reload the latest project and discard this session’s pending edits and undo history? Download pending edits first to keep a record.",
                    )
                  )
                    load(true);
                }}
              >
                Reload & discard pending edits
              </button>
            </>
          ) : (
            <>
              {[error, ...effects].filter(Boolean).join(" ") ||
                `Exported: ${exports.join(", ")}`}
              {!effects.length && (
                <button
                  aria-label="Dismiss notice"
                  onClick={() => {
                    setError("");
                    setExports([]);
                  }}
                >
                  ×
                </button>
              )}
            </>
          )}
        </div>
      )}
      <div className="workspace" inert={busy}>
        <aside className="slides-panel" aria-label="Slides">
          <div className="panel-heading">
            <h2>Slides</h2>
            <span>{model.manifest.slides.length}</span>
          </div>
          <ol>
            {model.manifest.slides.map((s, i) => (
              <li
                key={s.id}
                draggable={!typing}
                onDragStart={() => (dragSlide.current = s.id)}
                onDragOver={(e) => e.preventDefault()}
                onDrop={(e) => {
                  e.preventDefault();
                  if (dragSlide.current) reorder(dragSlide.current, i);
                }}
                className={slide === s.id ? "selected" : ""}
              >
                <button
                  className="slide-button"
                  aria-label={`Slide ${i + 1}: ${s.title || s.id}`}
                  aria-current={slide === s.id ? "true" : undefined}
                  onClick={() => pick(s.id)}
                >
                  <span className="thumbnail">
                    <iframe
                      tabIndex={-1}
                      title={`${s.title || s.id} thumbnail`}
                      sandbox="allow-scripts"
                      ref={(f) => {
                        if (f) thumbs.current.set(s.id, f);
                        else thumbs.current.delete(s.id);
                      }}
                      src={base!.frame_url}
                      style={{
                        width: model.manifest.width,
                        height: model.manifest.height,
                        transform: `scale(${160 / model.manifest.width})`,
                      }}
                    />
                  </span>
                  <span className="slide-caption">
                    <b>{String(i + 1).padStart(2, "0")}</b>
                    {s.title || s.id}
                  </span>
                </button>
                <div className="slide-order">
                  <button
                    aria-label={`Move ${s.id} up`}
                    disabled={i === 0}
                    onClick={() => reorder(s.id, i - 1)}
                  >
                    ↑
                  </button>
                  <button
                    aria-label={`Move ${s.id} down`}
                    disabled={i === model.manifest.slides.length - 1}
                    onClick={() => reorder(s.id, i + 1)}
                  >
                    ↓
                  </button>
                  <span>Drag to reorder</span>
                </div>
              </li>
            ))}
          </ol>
          <div className="source-note">
            HTML + CSS, always.
            <br />
            Your source stays yours.
          </div>
        </aside>
        <main className="center">
          <div className="canvas-heading">
            <span>
              <b>{active?.title || slide}</b>
              <small>
                {mode === "edit"
                  ? "Click to select · Double-click to edit text"
                  : "Presentation preview · Use arrow keys"}
              </small>
            </span>
            <div>
              <button
                aria-label="Zoom out"
                onClick={() => setZoom(Math.max(0.15, scale - 0.1))}
              >
                −
              </button>
              <select
                aria-label="Canvas zoom"
                value={zoom}
                onChange={(e) =>
                  setZoom(
                    e.target.value === "fit" ? "fit" : Number(e.target.value),
                  )
                }
              >
                <option value="fit">Fit · {Math.round(fit * 100)}%</option>
                {[
                  0.25,
                  0.5,
                  0.75,
                  1,
                  1.5,
                  2,
                  ...(typeof zoom === "number" &&
                  ![0.25, 0.5, 0.75, 1, 1.5, 2].includes(zoom)
                    ? [zoom]
                    : []),
                ].map((z) => (
                  <option key={z} value={z}>
                    {Math.round(z * 100)}%
                  </option>
                ))}
              </select>
              <button
                aria-label="Zoom in"
                onClick={() => setZoom(Math.min(2, scale + 0.1))}
              >
                +
              </button>
            </div>
          </div>
          <div className="canvas" ref={canvas}>
            <div
              className="canvas-sheet"
              style={{
                width: model.manifest.width * scale,
                height: model.manifest.height * scale,
              }}
            >
              <iframe
                ref={iframe}
                title="Presentation canvas"
                sandbox="allow-scripts"
                src={base!.frame_url}
                style={{
                  width: model.manifest.width,
                  height: model.manifest.height,
                  transform: `scale(${scale})`,
                }}
              />
            </div>
          </div>
          <div className="canvas-footer">
            <span>
              {model.manifest.width} × {model.manifest.height} px
            </span>
            <span>
              {mode === "edit" ? "EDITING" : "PREVIEW"} ·{" "}
              {model.manifest.slides.findIndex((s) => s.id === slide) + 1} /{" "}
              {model.manifest.slides.length}
            </span>
          </div>
          <section className="notes">
            <div>
              <h2>Speaker notes</h2>
              <span>Saved in deck.json · included in HTML and PPTX</span>
            </div>
            <Field
              key={slide}
              label="Notes"
              value={active?.notes || ""}
              multiline
              onCommit={(text) =>
                add("Edit notes", [{ type: "notes", slide, text }])
              }
            />
          </section>
        </main>
        <aside className="properties" aria-label="Properties">
          <div className="panel-heading">
            <h2>Properties</h2>
            <span>◈</span>
          </div>
          <div className="element-picker">
            <label htmlFor="element-list">Source element</label>
            <select
              id="element-list"
              value={selected || ""}
              onChange={(e) => {
                if ((e.target.value || null) === selected) return;
                setSelected(e.target.value || null);
                setSelection(null);
              }}
            >
              <option value="">Select on canvas…</option>
              {model.elements[slide].map((n) => (
                <option key={n.key} value={n.key}>
                  {n.tag} · {n.id || n.key}
                </option>
              ))}
            </select>
          </div>
          {node && selection ? (
            <div key={`${slide}/${selected}`} className="property-content">
              <div className="selection-title">
                <b>{node.tag.toUpperCase()}</b>
                <span>{node.id || "ID assigned on first saved edit"}</span>
                {selection.parentKey && (
                  <button
                    aria-label="Select parent element"
                    onClick={() => {
                      setSelected(selection.parentKey!);
                      setSelection(null);
                    }}
                  >
                    ↑ Parent
                  </button>
                )}
              </div>
              {selection.readOnly && (
                <p className="hint warning">{selection.readOnly}</p>
              )}
              <p className="hint">
                ● Purple dots mark GUI overrides. ↺ restores the authored style.
              </p>
              {node.tag !== "img" && (
                <details open>
                  <summary>Content</summary>
                  <Field
                    label="Text & inline formatting"
                    value={node.html}
                    multiline
                    disabled={!selection.textEditable || !!selection.readOnly}
                    reason="Only text with strong, em, b, i, u, s, code and br is editable. Containers with nested layouts remain intact."
                    onCommit={(html) => commitText(slide, node.key, html)}
                  />
                  <p className="hint">
                    Use &lt;strong&gt;bold&lt;/strong&gt;,
                    &lt;em&gt;emphasis&lt;/em&gt; and &lt;br&gt; for a line
                    break. Or double-click the canvas.
                  </p>
                </details>
              )}
              {node.tag === "img" && (
                <details open>
                  <summary>Image</summary>
                  <label className="upload">
                    Replace image
                    <input
                      type="file"
                      aria-label="Replace image"
                      accept="image/png,image/jpeg,image/webp,image/gif"
                      disabled={
                        !!selection.readOnly ||
                        "srcset" in node.attrs ||
                        "sizes" in node.attrs
                      }
                      onChange={async (e) => {
                        const f = e.target.files?.[0];
                        if (!f) return;
                        if (f.size > 10_000_000) {
                          setError("Choose an image smaller than 10 MB.");
                          return;
                        }
                        const reader = new FileReader();
                        reader.onload = () =>
                          add("Replace image", [
                            {
                              type: "image",
                              slide,
                              element: node.key,
                              data: String(reader.result),
                            },
                          ]);
                        reader.readAsDataURL(f);
                      }}
                    />
                  </label>
                  <p className="hint">
                    PNG, JPEG, WebP or GIF · up to 10 MB. Responsive srcset
                    images are read-only.
                  </p>
                  <Field
                    label="Alternative text"
                    disabled={!!selection.readOnly}
                    reason={selection.readOnly}
                    value={node.attrs.alt || ""}
                    multiline
                    onCommit={(text) =>
                      add("Edit alternative text", [
                        { type: "alt", slide, element: node.key, text },
                      ])
                    }
                  />
                  {prop("object-fit")}
                </details>
              )}
              {node.tag !== "img" && (
                <details open>
                  <summary>Typography</summary>
                  {[
                    "font-family",
                    "font-size",
                    "font-weight",
                    "font-style",
                    "text-decoration-line",
                    "color",
                    "text-align",
                  ].map(prop)}
                </details>
              )}
              <details open>
                <summary>Appearance & size</summary>
                {[
                  "background-color",
                  "width",
                  "height",
                  "padding",
                  "border-color",
                  "border-width",
                  "border-style",
                  "border-radius",
                ].map(prop)}
              </details>
              <details>
                <summary>Layout & spacing</summary>
                {[
                  "gap",
                  "row-gap",
                  "column-gap",
                  "align-items",
                  "justify-content",
                  "flex-direction",
                  "align-self",
                  "order",
                ].map(prop)}
              </details>
              <details>
                <summary>Position</summary>
                <p className="hint">
                  {selection.drag
                    ? "Coordinates are relative to the existing containing block. Transforms are preserved."
                    : selection.reason}
                </p>
                {["left", "top"].map(prop)}
              </details>
            </div>
          ) : (
            <div className="empty-state">
              <span>↖</span>
              <h3>Select an element</h3>
              <p>
                Click text, an image or a layout container to inspect its source
                and appearance.
              </p>
              <p>Use the source element menu for keyboard selection.</p>
            </div>
          )}
        </aside>
      </div>
    </div>
  );
}
createRoot(document.getElementById("root")!).render(<App />);
