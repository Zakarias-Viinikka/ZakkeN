document.querySelectorAll('[contenteditable]').forEach((el) => {
  el.addEventListener('input', () => {
    if (!el.textContent) el.replaceChildren();
  });
});

document.querySelectorAll('[data-title]').forEach((title) => {
  title.addEventListener('keydown', (e) => {
    if (e.key === 'Enter') {
      e.preventDefault();
      const first = document.querySelector('[data-block]');
      if (first) first.focus();
    }
  });
  title.addEventListener('paste', (e) => {
    e.preventDefault();
    const text = e.clipboardData.getData('text').replace(/\s*\n\s*/g, ' ');
    document.execCommand('insertText', false, text);
  });
});
