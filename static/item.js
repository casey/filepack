let audio = document.querySelector('audio');
let media = document.querySelector('img, video');
let next = document.querySelector('link[rel=next]');

let autoplay = sessionStorage.getItem('autoplay');
sessionStorage.removeItem('autoplay');

if (autoplay === location.href) {
  audio?.play();
}

audio?.addEventListener('ended', () => {
  if (next) {
    sessionStorage.setItem('autoplay', next.href);
    window.location = next.href;
  }
});

document.addEventListener('keydown', (event) => {
  if (event.altKey || event.ctrlKey || event.metaKey) {
    return;
  }

  if (event.key === 'f' && document.fullscreenEnabled) {
    if (document.fullscreenElement === null) {
      media.requestFullscreen();
    } else {
      document.exitFullscreen();
    }
  }
});
