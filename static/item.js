let audio = document.querySelector('audio');
let download = document.querySelector('a[download]');
let header = document.querySelector('header');
let media = document.querySelector('img, pre, video');
let next = document.querySelector('link[rel=next]');
let pre = document.querySelector('html.text pre');

let autoplay = sessionStorage.getItem('autoplay');
sessionStorage.removeItem('autoplay');

if (autoplay === location.href) {
  audio?.play();
}

function fit() {
  let top = header.getBoundingClientRect().bottom;
  let before = pre.getBoundingClientRect();
  let position = (top - before.top) / before.height;
  pre.style.removeProperty('--scale');
  let style = getComputedStyle(pre);
  let padding = parseFloat(style.paddingLeft) + parseFloat(style.paddingRight);
  let scale = (pre.clientWidth - padding) / (pre.scrollWidth - padding);
  if (scale < 1) {
    pre.style.setProperty('--scale', scale);
  }
  let after = pre.getBoundingClientRect();
  scrollBy(0, after.top - (top - position * after.height));
}

if (pre) {
  fit();
  addEventListener('resize', fit);
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

  if (event.key === 'd' && !event.repeat) {
    download.click();
  }

  if (event.key === 'f' && document.fullscreenEnabled) {
    if (document.fullscreenElement === null) {
      media.requestFullscreen();
    } else {
      document.exitFullscreen();
    }
  }
});
