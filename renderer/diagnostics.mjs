// Runs inside the deck's page. Keep browser inputs explicit and serializable.
export function inspectSlide({id, source, tolerance = 2, minimumFont = 18}) {
  const slide = [...document.querySelectorAll('[data-slide-id]')].find(s => s.dataset.slideId === id);
  const findings = [];
  const add = (severity, rule_id, element, message, measurements = {}) => findings.push({severity, rule_id, slide_id:id, element_id:element?.getAttribute('data-element-id') || element?.id || null, source, message, measurements});
  const seen = new Map();
  for (const el of slide.querySelectorAll('[data-element-id],[id]')) {
    for (const attr of ['data-element-id','id']) {
      const value = el.getAttribute(attr);
      if (!value) continue;
      const key = `${attr}:${value}`;
      if (seen.has(key)) add('error','element.duplicate_id',el,`Duplicate ${attr} "${value}" within this slide`, {attribute:attr,value});
      seen.set(key, el);
    }
  }
  const bounds = slide.getBoundingClientRect();
  for (const el of slide.querySelectorAll('*')) {
    if (!(el instanceof HTMLElement || el instanceof SVGElement)) continue;
    const style = getComputedStyle(el), box = el.getBoundingClientRect();
    if (!box.width || !box.height || style.visibility === 'hidden' || style.display === 'none' || el.closest('[hidden]')) continue;
    if (el instanceof HTMLImageElement && (!el.complete || el.naturalWidth === 0)) add('error','asset.broken_image',el,`Image could not decode: ${el.getAttribute('src')}`,{src:el.getAttribute('src')});
    const decorative = el.closest('[data-decksmith-overflow="decorative"][aria-hidden="true"]');
    // SVG internal geometry is intentionally not a layout oracle; inspect the SVG viewport.
    if ((!el.ownerSVGElement) && !decorative && (box.left < bounds.left-tolerance || box.top < bounds.top-tolerance || box.right > bounds.right+tolerance || box.bottom > bounds.bottom+tolerance)) {
      add('error','layout.out_of_bounds',el,'Element extends beyond the slide; move, resize, or explicitly mark decorative artwork', {x:box.left-bounds.left,y:box.top-bounds.top,width:box.width,height:box.height,slide_width:bounds.width,slide_height:bounds.height,tolerance});
    }
    const directText = [...el.childNodes].filter(n => n.nodeType === Node.TEXT_NODE && n.textContent.trim());
    if (directText.length && !decorative) {
      const size = parseFloat(style.fontSize);
      // Library slide folios are a known, intentionally small secondary label.
      if (size < minimumFont && !el.closest('.slide-footer')) add('warning','text.too_small',el,`Text is ${size}px; prefer at least ${minimumFont}px at logical size`,{font_size:size,minimum:minimumFont});
      if (el instanceof HTMLElement && (el.scrollWidth > el.clientWidth+tolerance || el.scrollHeight > el.clientHeight + (style.overflowY === 'visible' ? Math.max(tolerance, parseFloat(style.fontSize) * .3) : tolerance)) && style.display !== 'inline') {
        add('error','text.overflow',el,'Text exceeds its container; shorten it or increase available space', {scroll_width:el.scrollWidth,client_width:el.clientWidth,scroll_height:el.scrollHeight,client_height:el.clientHeight});
      } else {
        for (const node of directText) {
          const range = document.createRange(); range.selectNodeContents(node);
          const textBox = range.getBoundingClientRect();
          let parent = el;
          while (parent && parent !== slide) {
            const ps = getComputedStyle(parent), pb = parent.getBoundingClientRect();
            const clipX = ['hidden','clip','scroll','auto'].includes(ps.overflowX), clipY = ['hidden','clip','scroll','auto'].includes(ps.overflowY);
            if ((clipX && (textBox.left < pb.left-tolerance || textBox.right > pb.right+tolerance)) || (clipY && (textBox.top < pb.top-tolerance || textBox.bottom > pb.bottom+tolerance))) {
              add('error','text.clipped',el,'Text is clipped by an ancestor container', {container:parent.dataset.elementId || parent.tagName.toLowerCase()}); break;
            }
            parent = parent.parentElement;
          }
        }
      }
    }
  }
  return findings;
}
