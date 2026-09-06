let prev = document.querySelector('link[rel=prev]');
let next = document.querySelector('link[rel=next]');
let up = document.querySelector('link[rel=up]');

document.addEventListener('keydown', (event) => {
  if (event.altKey || event.ctrlKey || event.metaKey || event.shiftKey) {
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
