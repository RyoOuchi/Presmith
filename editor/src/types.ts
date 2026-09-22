export interface ElementInfo {
  key: string;
  id: string | null;
  tag: string;
  html: string;
  text_editable: boolean;
  read_only: string | null;
  attrs: Record<string, string>;
}
export interface Slide {
  id: string;
  source: string;
  title?: string;
  notes?: string;
}
export type Overrides = Record<string, Record<string, Record<string, string>>>;
export interface Project {
  manifest: {
    title: string;
    width: number;
    height: number;
    slides: Slide[];
    styles: string[];
    scripts: string[];
  };
  revision: string;
  frame_url: string;
  elements: Record<string, ElementInfo[]>;
  overrides: Overrides;
}
export type Operation =
  | { type: "text"; slide: string; element: string; html: string }
  | {
      type: "style";
      slide: string;
      element: string;
      property: string;
      value: string | null;
    }
  | { type: "image"; slide: string; element: string; data: string }
  | { type: "alt"; slide: string; element: string; text: string }
  | { type: "reorder"; order: string[] }
  | { type: "notes"; slide: string; text: string };
export interface Selection {
  key: string;
  tag: string;
  rect: { x: number; y: number; width: number; height: number };
  computed: Record<string, string>;
  blocked: Record<string, string>;
  resize: boolean;
  drag: boolean;
  reason: string;
  textEditable: boolean;
  readOnly?: string;
  parentKey?: string;
  effects: string[];
}
export function apply(base: Project, operations: Operation[]): Project {
  const p = structuredClone(base);
  // IDs such as "__proto__" are valid source IDs, never object prototypes.
  Object.setPrototypeOf(p.overrides, null);
  for (const elements of Object.values(p.overrides)) {
    Object.setPrototypeOf(elements, null);
    for (const props of Object.values(elements))
      Object.setPrototypeOf(props, null);
  }
  for (const o of operations) {
    if (o.type === "reorder")
      p.manifest.slides = o.order.map(
        (id) => p.manifest.slides.find((s) => s.id === id)!,
      );
    else if (o.type === "notes")
      p.manifest.slides.find((s) => s.id === o.slide)!.notes = o.text;
    else {
      const n = p.elements[o.slide]?.find((n) => n.key === o.element);
      if (!n) continue;
      if (o.type === "text") n.html = o.html;
      if (o.type === "alt") n.attrs.alt = o.text;
      if (o.type === "image") n.attrs.src = o.data;
      if (o.type === "style") {
        const props = ((p.overrides[o.slide] ??= Object.create(null))[
          o.element
        ] ??= Object.create(null));
        if (o.value === null) delete props[o.property];
        else props[o.property] = o.value;
      }
    }
  }
  return p;
}
export const properties = [
  "font-family",
  "font-size",
  "font-weight",
  "font-style",
  "text-decoration-line",
  "color",
  "text-align",
  "background-color",
  "width",
  "height",
  "padding",
  "gap",
  "row-gap",
  "column-gap",
  "border-color",
  "border-width",
  "border-style",
  "border-radius",
  "object-fit",
  "left",
  "top",
  "align-items",
  "align-self",
  "justify-content",
  "flex-direction",
  "order",
];
