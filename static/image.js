function fit() {
  let scale = Math.min(
    document.documentElement.clientWidth / width,
    document.documentElement.clientHeight / height,
  );
  img.style.width = `${width * scale}px`;
}

document.documentElement.classList.add('js');

let img = document.querySelector('img');
let width = Number(img.getAttribute('width'));
let height = Number(img.getAttribute('height'));
let ratio = devicePixelRatio;

fit();

addEventListener('resize', () => {
  if (devicePixelRatio === ratio) {
    fit();
  } else {
    ratio = devicePixelRatio;
  }
});
