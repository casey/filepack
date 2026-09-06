let media = document.querySelector('img, video');
let next = document.querySelector('link[rel=next]');
let prev = document.querySelector('link[rel=prev]');
let shortcuts = document.querySelector('#shortcuts');
let up = document.querySelector('link[rel=up]');

document.addEventListener('keydown', (event) => {
  if (event.altKey || event.ctrlKey || event.metaKey) {
    return;
  }

  if (event.key === '?') {
    shortcuts.togglePopover();
    return;
  }

  if (event.key === 'f' && document.fullscreenEnabled) {
    if (document.fullscreenElement === null) {
      media.requestFullscreen();
    } else {
      document.exitFullscreen();
    }
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
