let media = document.querySelector('img, video');

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
