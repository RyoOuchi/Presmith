// Live demonstration with a repeatable capture state.
function setSize(slide, size) {
  const value = Math.min(88, Math.max(48, Number(size)));
  slide.querySelector('#headline-scale').value = String(value);
  slide.querySelector('#headline-size').textContent = `${value}px`;
  slide.querySelector('[data-element-id="specimen-text"]').style.fontSize = `${value}px`;
}
function wireSlider(slide) {
  const input = slide.querySelector('#headline-scale');
  input.addEventListener('input', () => setSize(slide, input.value));
  setSize(slide, input.value);
}
Decksmith.register('interactive', {
  init(slide) {
    wireSlider(slide);
  },
  export(slide) {
    setSize(slide, 72);
  }
});
