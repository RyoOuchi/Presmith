import {
  properties,
  type ElementInfo,
  type Overrides,
  type Selection,
} from "./types";
declare global {
  interface Window {
    Decksmith: {
      select(id: string): void;
      ready(id: string): Promise<void>;
      current(): string;
    };
  }
}
// This script is injected only into a sandboxed editor srcdoc, before author scripts.
const refs = new Map<string, HTMLElement>();
const generated = new Set<string>();
const initial = new WeakMap<
  HTMLElement,
  { html: string; style: string | null; src: string | null }
>();
for (const el of document.querySelectorAll<HTMLElement>("[data-editor-key]")) {
  const slide = el.closest<HTMLElement>("[data-slide-id]")!.dataset.slideId!;
  refs.set(`${slide}/${el.dataset.editorKey}`, el);
  initial.set(el, {
    html: el.innerHTML,
    style: el.getAttribute("style"),
    src: el.getAttribute("src"),
  });
  if (!el.dataset.elementId) {
    generated.add(`${slide}/${el.dataset.editorKey}`);
    el.dataset.elementId = el.dataset.editorKey;
  }
}
function sourceHTML(el: HTMLElement) {
  const clone = el.cloneNode(true) as HTMLElement;
  for (const child of clone.querySelectorAll<HTMLElement>(
    "[data-editor-key]",
  )) {
    const k = child.dataset.editorKey!;
    if (
      generated.has(
        `${el.closest<HTMLElement>("[data-slide-id]")?.dataset.slideId}/${k}`,
      )
    )
      child.removeAttribute("data-element-id");
    child.removeAttribute("data-editor-key");
  }
  return clone.innerHTML;
}
for (const el of refs.values()) initial.get(el)!.html = sourceHTML(el);
const environment = document.createElement("style");
environment.textContent =
  "#deck-stage{left:0!important;top:0!important;transform:none!important}#deck-controls{display:none!important}html,body{overflow:hidden!important}";
document.head.append(environment);
const css = document.createElement("style");
document.head.append(css);
const existing = document.querySelector<HTMLLinkElement>(
  'link[href="styles/editor.css"]',
);
existing?.remove();
const overlay = document.createElement("div");
overlay.id = "editor-selection";
Object.assign(overlay.style, {
  position: "fixed",
  outline: "2px solid #7561f5",
  pointerEvents: "none",
  zIndex: "2147483646",
  display: "none",
  boxSizing: "border-box",
});
document.body.append(overlay);
const label = document.createElement("span");
Object.assign(label.style, {
  position: "absolute",
  top: "-25px",
  left: "-2px",
  background: "#6752dc",
  color: "#fff",
  font: "12px system-ui",
  padding: "4px 7px",
  whiteSpace: "nowrap",
  borderRadius: "4px 4px 0 0",
});
overlay.append(label);
let slide = "",
  key: string | null = null,
  mode = "edit",
  elements: ElementInfo[] = [],
  overrides: Overrides = {},
  editing: HTMLElement | null = null,
  composing = false,
  gesture = false,
  validating = false,
  last = "",
  effects: string[] = [];
const send = (type: string, extra: object = {}) =>
  parent.postMessage({ decksmith: true, type, ...extra }, "*");
const selected = () => (key ? refs.get(`${slide}/${key}`) : undefined);
const currentInfo = () => elements.find((n) => n.key === key);
function layout(el: HTMLElement) {
  const c = getComputedStyle(el);
  let transformed = false;
  for (
    let p: HTMLElement | null = el;
    p && p.id !== "deck-stage";
    p = p.parentElement
  ) {
    const t = getComputedStyle(p).transform;
    if (t !== "none") {
      const m = new DOMMatrix(t);
      if (m.a !== 1 || m.b !== 0 || m.c !== 0 || m.d !== 1) transformed = true;
    }
  }
  const resize =
    !transformed &&
    !["inline", "contents", "none"].includes(c.display) &&
    !["TABLE", "TR", "TD", "TH"].includes(el.tagName);
  const typed = (
    el as HTMLElement & {
      computedStyleMap?: () => Map<string, { toString(): string }>;
    }
  ).computedStyleMap?.();
  const drag =
    !transformed &&
    c.position === "absolute" &&
    typed?.get("right")?.toString() === "auto" &&
    typed?.get("bottom")?.toString() === "auto" &&
    typed?.get("left")?.toString() !== "auto" &&
    typed?.get("top")?.toString() !== "auto" &&
    !!el.offsetParent;
  return {
    resize,
    drag,
    reason: transformed
      ? "Geometry is read-only for scaled, rotated or skewed elements/ancestors."
      : !drag
        ? "Move requires absolute positioning with left/top anchors and no right/bottom constraint."
        : "",
  };
}
function inlineConflict(style: string, property: string) {
  return style.split(";").some((d) => {
    const p = d.split(":")[0].trim().toLowerCase();
    return (
      p === property ||
      p === "all" ||
      (p === "font" && property.startsWith("font-")) ||
      ([
        "border",
        "padding",
        "background",
        "flex",
        "gap",
        "inset",
        "text-decoration",
      ].includes(p) &&
        (property.startsWith(p + "-") ||
          (p === "inset" && ["left", "top"].includes(property))))
    );
  });
}
function canonical(html: string) {
  const t = document.createElement("template");
  t.innerHTML = html;
  return t.innerHTML;
}
function inspect(): Selection | undefined {
  const el = selected(),
    info = currentInfo();
  if (!el || !info) return;
  if (!el.isConnected) {
    key = null;
    send("readonly", {
      message:
        "The parent text was replaced in this session. Save and reload to select its new inline descendants.",
    });
    return;
  }
  const c = getComputedStyle(el),
    r = el.getBoundingClientRect(),
    geo = layout(el),
    computed: Record<string, string> = {},
    blocked: Record<string, string> = {};
  for (const p of properties) {
    computed[p] = c.getPropertyValue(p);
    if (inlineConflict(info.attrs.style || "", p))
      blocked[p] =
        "Controlled by inline CSS. Remove or edit it in the source first.";
  }
  for (const p of ["left", "top"]) if (!geo.drag) blocked[p] = geo.reason;
  for (const p of ["width", "height"])
    if (!geo.resize)
      blocked[p] =
        "Sizing requires a block, flex/grid item or image without scaled/rotated ancestors.";
  for (const p of [
    "gap",
    "row-gap",
    "column-gap",
    "align-items",
    "justify-content",
  ])
    if (!["flex", "inline-flex", "grid", "inline-grid"].includes(c.display))
      blocked[p] = "Select a flex or grid container.";
  if (!c.display.includes("flex"))
    blocked["flex-direction"] = "Select a flex container.";
  if (!getComputedStyle(el.parentElement!).display.match(/flex|grid/))
    for (const p of ["order", "align-self"])
      blocked[p] = "Select a flex or grid item.";
  if (info.tag !== "img") blocked["object-fit"] = "Select an image.";
  const changedIdentity =
    el.dataset.elementId !== (info.id || info.key) ||
    el.closest<HTMLElement>("[data-slide-id]")?.dataset.slideId !== slide ||
    (info.tag === "img" && el.getAttribute("src") !== initial.get(el)?.src);
  const readOnly =
    info.read_only ||
    (changedIdentity
      ? "Runtime script changed this element’s source identity or image. Edit its source or script."
      : !gesture &&
          (el.getAttribute("style") || "").trim() !==
            (initial.get(el)?.style || "").trim()
        ? `Runtime script changed this element’s inline style (${el.getAttribute("style") || "empty"}). Edit its source or script.`
        : undefined);
  return {
    key: key!,
    tag: info.tag,
    rect: { x: r.x, y: r.y, width: r.width, height: r.height },
    computed,
    blocked,
    ...geo,
    textEditable: info.text_editable && sourceHTML(el) === canonical(info.html),
    readOnly,
    parentKey:
      el.parentElement?.closest<HTMLElement>("[data-editor-key]")?.dataset
        .editorKey,
    effects,
  };
}
function measure() {
  const s = inspect();
  if (!s || mode !== "edit" || editing || validating) {
    overlay.style.display = "none";
    return;
  }
  Object.assign(overlay.style, {
    display: "block",
    left: `${s.rect.x}px`,
    top: `${s.rect.y}px`,
    width: `${s.rect.width}px`,
    height: `${s.rect.height}px`,
  });
  label.textContent = s.tag + " · " + s.key;
  for (const h of overlay.querySelectorAll<HTMLElement>("[data-handle]"))
    h.style.display =
      (h.dataset.handle === "move" ? s.drag : s.resize) && !s.readOnly
        ? "block"
        : "none";
  const next = JSON.stringify(s);
  if (next !== last) {
    last = next;
    send("selection", { selection: s });
  }
}
function choose(el: HTMLElement) {
  const source = el.closest<HTMLElement>("[data-editor-key]");
  const candidate = source?.dataset.editorKey;
  const info = elements.find((n) => n.key === candidate);
  const indirect =
    source &&
    el !== source &&
    !(info?.text_editable && sourceHTML(source) === canonical(info.html));
  if (
    indirect ||
    !source ||
    refs.get(`${slide}/${candidate}`) !== source ||
    !elements.some((n) => n.key === candidate)
  ) {
    key = null;
    overlay.style.display = "none";
    send("readonly", {
      message:
        "This node was generated by a script or has no supported source mapping. Edit its source code.",
    });
    return;
  }
  key = candidate!;
  last = "";
  measure();
}
function rules() {
  css.textContent = Object.entries(overrides)
    .flatMap(([s, es]) =>
      Object.entries(es).map(
        ([e, ps]) =>
          `[data-slide-id="${s}"] [data-element-id="${e}"]{${Object.entries(ps)
            .sort(([a], [b]) => a.localeCompare(b))
            .map(([p, v]) => `${p}:${v}`)
            .join(";")}}`,
      ),
    )
    .join("\n");
}
function checkEffects(emit = true) {
  effects = [];
  for (const [s, es] of Object.entries(overrides))
    for (const [e, ps] of Object.entries(es)) {
      const el = refs.get(`${s}/${e}`);
      if (!el || el.closest<HTMLElement>("[data-slide-id]")?.hidden) continue;
      const entries = Object.entries(ps).sort(([a], [b]) => a.localeCompare(b));
      const actual = Object.fromEntries(
        entries.map(([p]) => [p, getComputedStyle(el).getPropertyValue(p)]),
      );
      const previous = el.getAttribute("style");
      // Compare the complete declaration set so GUI shorthand/longhand combinations
      // have identical cascade order during validation and in persisted editor.css.
      for (const [p, v] of entries) el.style.setProperty(p, v, "important");
      for (const [p, v] of entries) {
        if (
          ["width", "height"].includes(p) &&
          Math.abs(parseFloat(actual[p]) - parseFloat(v)) > 0.5
        )
          effects.push(
            `${e}: ${p} is constrained by authored min/max sizing or flex layout (${actual[p]}). Reset it or adjust the source constraints.`,
          );
        const desired = getComputedStyle(el).getPropertyValue(p);
        if (actual[p] !== desired)
          effects.push(
            `${e}: ${p} is blocked by authored CSS (${actual[p]}). Reset the override or edit the source rule.`,
          );
      }
      if (previous === null) el.removeAttribute("style");
      else el.setAttribute("style", previous);
    }
  if (emit) send("effects", { effects });
  return effects;
}
window.addEventListener("message", async (event) => {
  if (event.source !== parent || !event.data?.decksmith) return;
  const d = event.data;
  if (d.type === "sync") {
    if (editing || gesture) return;
    slide = d.slide;
    mode = d.mode;
    elements = d.elements;
    const scale = Number.isFinite(d.scale) && d.scale > 0 ? d.scale : 1;
    overlay.style.outlineWidth = `${2 / scale}px`;
    label.style.font = `${11 / scale}px system-ui`;
    label.style.padding = `${3 / scale}px ${6 / scale}px`;
    label.style.top = `${-22 / scale}px`;
    for (const h of overlay.querySelectorAll<HTMLElement>("[data-handle]")) {
      const k = h.dataset.handle;
      h.style.width = `${(k === "move" ? 24 : 10) / scale}px`;
      h.style.height = `${(k === "move" ? 22 : 10) / scale}px`;
      h.style.borderWidth = `${2 / scale}px`;
      h.style.fontSize = `${14 / scale}px`;
      h.style.right =
        k === "move" ? "auto" : k === "s" ? "50%" : `${-5 / scale}px`;
      if (k === "move") {
        h.style.left = `${-26 / scale}px`;
        h.style.top = `${-22 / scale}px`;
      } else h.style.bottom = k === "e" ? "50%" : `${-5 / scale}px`;
    }
    overrides = d.overrides;
    key = d.key ?? null;
    for (const [s, nodes] of Object.entries(
      d.allElements as Record<string, ElementInfo[]>,
    ))
      for (const n of nodes) {
        const el = refs.get(`${s}/${n.key}`);
        if (!el) continue;
        const old = initial.get(el)!;
        // Runtime-generated content is never copied into source or normalized away.
        if (
          n.text_editable &&
          sourceHTML(el) === old.html &&
          canonical(n.html) !== old.html
        ) {
          el.innerHTML = n.html;
          old.html = canonical(n.html);
        }
        if (n.tag === "img") {
          if (n.attrs.src !== undefined && el.getAttribute("src") === old.src) {
            el.setAttribute("src", n.attrs.src);
            old.src = n.attrs.src;
          }
          if (n.attrs.alt !== undefined) el.setAttribute("alt", n.attrs.alt);
        }
      }
    rules();
    try {
      window.Decksmith.select(slide);
      await window.Decksmith.ready(slide);
    } catch (e) {
      send("error", { message: String(e) });
    }
    checkEffects();
    last = "";
    measure();
  }
  if (d.type === "select") {
    key = d.key;
    last = "";
    measure();
  }
  if (d.type === "validate") {
    validating = true;
    const all: string[] = [];
    const current = slide;
    try {
      for (const id of Object.keys(overrides)) {
        await window.Decksmith.ready(id);
        all.push(...checkEffects(false).map((e) => `${id}: ${e}`));
      }
      await window.Decksmith.ready(current);
      send("validated", { id: d.id, effects: all });
    } catch (e) {
      send("validated", { id: d.id, error: String(e) });
    } finally {
      validating = false;
      window.Decksmith.select(current);
      last = "";
      measure();
    }
  }
});
document.addEventListener(
  "click",
  (e) => {
    if (!e.isTrusted) return;
    if (mode !== "edit" || editing || overlay.contains(e.target as Node))
      return;
    e.preventDefault();
    e.stopImmediatePropagation();
    choose(e.target as HTMLElement);
  },
  true,
);
document.addEventListener(
  "dblclick",
  (e) => {
    if (!e.isTrusted) return;
    if (mode !== "edit") return;
    e.preventDefault();
    e.stopImmediatePropagation();
    choose(e.target as HTMLElement);
    const el = selected(),
      s = inspect();
    if (!el || !s?.textEditable || s.readOnly) return;
    editing = el;
    el.contentEditable = "true";
    el.focus();
    overlay.style.display = "none";
    send("editing", { active: true });
  },
  true,
);
function normalizedHTML(el: HTMLElement) {
  const clone = document.createElement("div");
  clone.innerHTML = sourceHTML(el);
  // Chromium's Enter/paste normalization is deliberately limited to line breaks.
  for (const div of [...clone.querySelectorAll("div,p")]) {
    const br = document.createElement("br");
    div.before(br);
    div.replaceWith(...div.childNodes);
  }
  return clone.innerHTML;
}
function finish() {
  if (!editing || composing) return;
  const el = editing;
  editing = null;
  el.removeAttribute("contenteditable");
  const html = normalizedHTML(el);
  el.innerHTML = html;
  const info = currentInfo()!;
  if (html !== info.html) {
    send("text", { slide, element: key, html });
    initial.get(el)!.html = html;
  }
  send("editing", { active: false });
  last = "";
  measure();
}
document.addEventListener("compositionstart", () => (composing = true), true);
document.addEventListener(
  "compositionend",
  () => {
    composing = false;
    setTimeout(() => {
      if (editing && document.activeElement !== editing) finish();
    }, 0);
  },
  true,
);
document.addEventListener("focusout", () => finish(), true);
document.addEventListener(
  "paste",
  (e) => {
    if (!editing) return;
    e.preventDefault();
    document.execCommand(
      "insertText",
      false,
      e.clipboardData?.getData("text/plain") || "",
    );
  },
  true,
);
document.addEventListener(
  "keydown",
  (e) => {
    if (mode !== "edit") return;
    if (editing) {
      if (e.key === "Escape" && !composing) {
        e.preventDefault();
        editing.blur();
      }
      e.stopPropagation();
      return;
    }
    e.stopImmediatePropagation();
    if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === "z") {
      e.preventDefault();
      send(e.shiftKey ? "redo" : "undo");
    }
    if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === "s") {
      e.preventDefault();
      send("savehint");
    }
  },
  true,
);
for (const kind of ["move", "e", "s", "se"]) {
  const handle = document.createElement("button");
  handle.dataset.handle = kind;
  handle.setAttribute(
    "aria-label",
    kind === "move" ? "Move element" : `Resize ${kind}`,
  );
  Object.assign(handle.style, {
    position: "absolute",
    pointerEvents: "auto",
    width: kind === "move" ? "27px" : "12px",
    height: kind === "move" ? "24px" : "12px",
    border: "2px solid #6752dc",
    background: "#fff",
    color: "#6752dc",
    padding: "0",
    cursor: kind === "move" ? "move" : `${kind}-resize`,
    zIndex: "1",
    font: "14px system-ui",
  });
  if (kind === "move") {
    handle.textContent = "✥";
    handle.style.right = "0";
    handle.style.top = "-25px";
  } else {
    handle.style.right = "-7px";
    handle.style.bottom = kind === "e" ? "50%" : "-7px";
    if (kind === "s") handle.style.right = "50%";
  }
  overlay.append(handle);
  handle.addEventListener("pointerdown", (e) => {
    e.preventDefault();
    e.stopPropagation();
    const el = selected(),
      s = inspect();
    if (!el || !s || s.readOnly) return;
    const props =
      kind === "move"
        ? ["left", "top"]
        : kind === "e"
          ? ["width"]
          : kind === "s"
            ? ["height"]
            : ["width", "height"];
    if (props.some((p) => s.blocked[p])) return;
    const old = el.getAttribute("style"),
      x = e.clientX,
      y = e.clientY,
      values = Object.fromEntries(
        props.map((p) => [p, parseFloat(s.computed[p])]),
      );
    if (Object.values(values).some((v) => !Number.isFinite(v))) return;
    gesture = true;
    handle.setPointerCapture(e.pointerId);
    let result: Record<string, string> = {};
    const move = (event: PointerEvent) => {
      result = {};
      for (const p of props) {
        const delta = ["left", "width"].includes(p)
          ? event.clientX - x
          : event.clientY - y;
        const value = Math.round((values[p] + delta) * 10) / 10;
        result[p] =
          `${["width", "height"].includes(p) ? Math.max(1, Math.min(8192, value)) : Math.max(-8192, Math.min(8192, value))}px`;
        el.style.setProperty(p, result[p]);
      }
      measure();
    };
    const done = (event: PointerEvent) => {
      handle.removeEventListener("pointermove", move);
      handle.removeEventListener("pointerup", done);
      handle.removeEventListener("pointercancel", cancel);
      if (old === null) el.removeAttribute("style");
      else el.setAttribute("style", old);
      gesture = false;
      if (event.type !== "pointercancel" && Object.keys(result).length)
        send("geometry", { slide, element: key, values: result });
      measure();
    };
    const cancel = (event: PointerEvent) => done(event);
    handle.addEventListener("pointermove", move);
    handle.addEventListener("pointerup", done);
    handle.addEventListener("pointercancel", cancel);
  });
}
function loop() {
  if (!gesture) measure();
  requestAnimationFrame(loop);
}
requestAnimationFrame(loop);
window.addEventListener("load", () => send("ready"));
document.addEventListener(
  "error",
  (e) => {
    const el = e.target;
    if (el instanceof HTMLImageElement)
      send("error", {
        message: `Image could not load: ${el.getAttribute("src")}. Check the file under assets/.`,
      });
  },
  true,
);
document.addEventListener("decksmith:slidechange", () => {
  if (mode === "preview" && !validating)
    send("slide", { slide: window.Decksmith.current() });
});
