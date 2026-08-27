let prev = document.querySelector('link[rel=prev]');
let next = document.querySelector('link[rel=next]');
let up = document.querySelector('link[rel=up]');

document.addEventListener('keydown', (event) => {
  if (event.altKey || event.ctrlKey || event.metaKey || event.shiftKey) {
    return;
  }

  let link = event.key === 'ArrowLeft' ? prev
    : event.key === 'ArrowRight' ? next
    : event.key === 'ArrowUp' ? up
    : null;

  if (link !== null) {
    window.location = link.href;
  }
});
