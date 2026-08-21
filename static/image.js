let img = document.querySelector('img');
let width = Number(img.getAttribute('width'));
let height = Number(img.getAttribute('height'));

let scale = Math.min(
  document.documentElement.clientWidth / width,
  document.documentElement.clientHeight / height,
);

document.documentElement.classList.add('js');

img.style.width = `${width * scale}px`;
