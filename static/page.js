document
  .querySelector(`nav a[href="${location.pathname}"]`)
  ?.setAttribute('aria-current', 'page');

let next = document.querySelector('link[rel=next]');
let prev = document.querySelector('link[rel=prev]');
let shortcuts = document.querySelector('#shortcuts');
let up = document.querySelector('link[rel=up]');

document.addEventListener('keydown', (event) => {
  if (event.altKey || event.ctrlKey || event.metaKey) {
    return;
  }

  if (event.key === '?' && shortcuts !== null) {
    shortcuts.togglePopover();
    return;
  }

  let link = event.key === 'p' ? prev
    : event.key === 'n' ? next
    : event.key === 'u' ? up
    : null;

  if (link !== null) {
    window.location = link.href;
  }
});
