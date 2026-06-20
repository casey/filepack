function html([source]) {
  let template = document.createElement('template');
  template.innerHTML = source;
  return template;
}

function css([source]) {
  let sheet = new CSSStyleSheet();
  sheet.replaceSync(source);
  return sheet;
}

class PlayButton extends HTMLElement {
  static styles = css`
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
  `;

  static template = html`
    <button>▶</button>
    <audio></audio>
  `;

  static get observedAttributes() {
    return ['src'];
  }

  constructor() {
    super();

    let shadow = this.attachShadow({ mode: 'open' });
    shadow.adoptedStyleSheets = [this.constructor.styles];
    shadow.append(this.constructor.template.content.cloneNode(true));

    let audio = shadow.querySelector('audio');
    let button = shadow.querySelector('button');

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

    if (name === 'src') {
      this.shadowRoot.querySelector('audio').src = value;
    }
  }
}

customElements.define('play-button', PlayButton);
