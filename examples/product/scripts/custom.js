Decksmith.register('lab', {
  init(slide) {
    const input = slide.querySelector('#adoption');
    const update = () => {
      const value = Number(input.value);
      const height = value / 90 * 156;
      slide.querySelector('#adoption-label').textContent = `${value}%`;
      slide.querySelector('#adoption-bar').setAttribute('height', height);
      slide.querySelector('#adoption-bar').setAttribute('y', 200 - height);
      const label = slide.querySelector('#chart-value');
      label.textContent = `${value}%`; label.setAttribute('y', 186 - height);
      slide.querySelector('svg').setAttribute('aria-label', `Illustrative adoption is ${value} percent`);
    };
    input.addEventListener('input', update); update();
  },
  export(slide) {
    const input = slide.querySelector('#adoption');
    input.value = '60';
    input.dispatchEvent(new Event('input'));
  }
});
