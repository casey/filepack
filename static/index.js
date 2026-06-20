const style = new CSSStyleSheet()

style.replaceSync(`
button {
  background: none;
  border: none;
  color: inherit;
  cursor: pointer;
  height: 1rem;
  padding: 0;
  width: 1rem;
}

button:hover {
  text-shadow: 0 0 5px #fff;
}
`);

const template = document.createElement('template')
template.innerHTML = `
  <button>▶</button>
  <audio></audio>
`;

class PlayButton extends HTMLElement {
  static get observedAttributes() {
    return ['src'];
  }

  constructor() {
    super();

    const shadow = this.attachShadow({ mode: 'open' });
    shadow.adoptedStyleSheets = [style];
    shadow.append(template.content.cloneNode(true));

    const audio = shadow.querySelector('audio');
    const button = shadow.querySelector('button');

    button.addEventListener('click', () => {
      if (audio.paused) {
        audio.play();
      } else {
        audio.pause();
      }
    });

    audio.addEventListener('play', () => {
      button.textContent = '⏸'
    });

    audio.addEventListener('pause', () => {
      button.textContent = '▶'
    });
  }

  attributeChangedCallback(name, old, value) {
    if (value === old) {
      return;
    }

    if (name == 'src') {
      this.shadowRoot.querySelector('audio').src = value;
    }
  }
}

customElements.define('play-button', PlayButton);
