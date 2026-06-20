function html([source]) {
  const template = document.createElement('template');
  template.innerHTML = source;
  return template;
}

class PlayButton extends HTMLElement {
  static template = html`<button>▶</button><audio></audio>`;

  static get observedAttributes() {
    return ['src'];
  }

  connectedCallback() {
    if (this.querySelector('button')) {
      return;
    }

    this.append(this.constructor.template.content.cloneNode(true));

    const audio = this.querySelector('audio');
    const button = this.querySelector('button');

    audio.src = this.getAttribute('src');

    button.addEventListener('click', () => {
      if (audio.paused) {
        audio.play();
      } else {
        audio.pause();
      }
    });

    audio.addEventListener('play', () => {
      button.textContent = '⏸';
    });

    audio.addEventListener('pause', () => {
      button.textContent = '▶';
    });
  }

  attributeChangedCallback(name, old, value) {
    if (value === old) {
      return;
    }

    if (name == 'src') {
      const audio = this.querySelector('audio');
      if (audio) {
        audio.src = value;
      }
    }
  }
}

customElements.define('play-button', PlayButton);
