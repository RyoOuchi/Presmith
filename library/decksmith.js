/* Decksmith runtime contract v1. No dependencies, no network. */
(() => {
  'use strict';
  const manifest = JSON.parse(document.getElementById('deck-manifest').textContent);
  const slides = [...document.querySelectorAll('#deck-stage > [data-slide-id]')];
  const registered = new Map();
  let index = 0, exporting = false;
  const bounded = (promise, label, ms = 10000) => {
    let timer;
    return Promise.race([promise, new Promise((_, reject) => { timer = setTimeout(() => reject(new Error(`Readiness timeout: ${label} (${ms}ms)`)), ms); })]).finally(() => clearTimeout(timer));
  };
  function resize() {
    document.documentElement.style.setProperty('--deck-scale', Math.max(.01, Math.min(innerWidth / manifest.width, (innerHeight - 84) / manifest.height)));
  }
  function select(id) {
    const next = slides.findIndex(s => s.dataset.slideId === id);
    if (next < 0) throw new Error(`Unknown slide: ${id}`);
    const changed = index !== next;
    index = next;
    slides.forEach((slide, i) => { slide.hidden = i !== index; slide.inert = i !== index; slide.setAttribute('aria-hidden', String(i !== index)); });
    if (slides.some(s => s.hidden && s.contains(document.activeElement))) document.activeElement.blur();
    if (location.hash.slice(1) !== id) history.replaceState(null, '', `#${id}`);
    document.getElementById('deck-position').textContent = `${index + 1} / ${slides.length}`;
    document.getElementById('deck-prev').disabled = index === 0;
    document.getElementById('deck-next').disabled = index === slides.length - 1;
    if (changed) document.dispatchEvent(new CustomEvent('decksmith:slidechange', {detail:{id, index}}));
    return id;
  }
  const step = delta => select(slides[Math.max(0, Math.min(slides.length - 1, index + delta))].dataset.slideId);
  async function ready(id = slides[index].dataset.slideId) {
    select(id);
    const slide = slides[index];
    const entry = registered.get(id);
    if (entry) {
      entry.promise ??= Promise.resolve().then(() => entry.init?.(slide));
      await bounded(entry.promise, `${id} initialization`);
      if (exporting) await bounded(Promise.resolve().then(() => entry.export?.(slide)), `${id} export state`);
    }
    await bounded(document.fonts.ready, 'fonts');
    await bounded(Promise.all([...slide.querySelectorAll('img')].map(img => {
      img.loading = 'eager';
      if (img.complete) return img.decode().catch(() => {});
      return new Promise(resolve => { img.addEventListener('load', resolve, {once:true}); img.addEventListener('error', resolve, {once:true}); });
    })), `${id} images`);
    await new Promise(resolve => requestAnimationFrame(() => requestAnimationFrame(resolve)));
  }
  window.Decksmith = Object.freeze({
    version:1,
    listSlides:() => manifest.slides.map(({id,title,source}, i) => ({id,title,source,index:i})),
    select,
    current:() => slides[index].dataset.slideId,
    setExportMode:enabled => {
      exporting = Boolean(enabled); document.documentElement.toggleAttribute('data-decksmith-export', exporting);
      if (exporting) { for (const a of document.getAnimations()) a.cancel(); for (const media of document.querySelectorAll('video,audio')) { media.pause(); media.currentTime = 0; } }
      resize();
    },
    ready,
    register:(id, hooks) => {
      if (!manifest.slides.some(s => s.id === id)) throw new Error(`Cannot register missing slide ${id}`);
      if (registered.has(id)) throw new Error(`Duplicate initialization for ${id}`);
      registered.set(id, hooks);
    },
  });
  document.getElementById('deck-prev').onclick = () => step(-1);
  document.getElementById('deck-next').onclick = () => step(1);
  document.getElementById('deck-fullscreen').onclick = async () => {
    try { if (document.fullscreenElement) await document.exitFullscreen(); else await document.documentElement.requestFullscreen(); }
    catch { document.getElementById('deck-fullscreen').textContent = 'Fullscreen unavailable'; }
  };
  document.addEventListener('keydown', e => {
    if (e.defaultPrevented || e.altKey || e.ctrlKey || e.metaKey || e.shiftKey || e.target.closest('input,textarea,select,button,a,[contenteditable]:not([contenteditable="false"]),[role="slider"]')) return;
    if (['ArrowRight','ArrowDown','PageDown',' '].includes(e.key)) { e.preventDefault(); step(1); }
    if (['ArrowLeft','ArrowUp','PageUp'].includes(e.key)) { e.preventDefault(); step(-1); }
    if (e.key === 'Home') { e.preventDefault(); select(slides[0].dataset.slideId); }
    if (e.key === 'End') { e.preventDefault(); select(slides.at(-1).dataset.slideId); }
    if (e.key.toLowerCase() === 'f') document.getElementById('deck-fullscreen').click();
  });
  window.addEventListener('hashchange', () => {
    const id = location.hash.slice(1);
    if (slides.some(s => s.dataset.slideId === id)) select(id);
  });
  window.addEventListener('resize', resize);
  // Author scripts register synchronously after this script, before DOMContentLoaded.
  document.addEventListener('DOMContentLoaded', () => {
    select(slides.some(s => s.dataset.slideId === location.hash.slice(1)) ? location.hash.slice(1) : slides[0].dataset.slideId);
    resize();
    const initialize = () => { ready().catch(error => { setTimeout(() => { throw error; }); }); };
    document.addEventListener('decksmith:slidechange', () => {
      const entry = registered.get(slides[index].dataset.slideId);
      if (entry && !entry.promise) initialize();
    });
    initialize();
  }, {once:true});
})();
