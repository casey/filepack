let prev = document.querySelector('link[rel=prev]');
let next = document.querySelector('link[rel=next]');

document.addEventListener('keydown', (event) => {
  if (event.altKey || event.ctrlKey || event.metaKey || event.shiftKey) {
    return;
  }

  let link = event.key === 'ArrowLeft' ? prev
    : event.key === 'ArrowRight' ? next
    : null;

  if (link !== null) {
    window.location = link.href;
  }
});
