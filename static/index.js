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

    let context;
    let gain;
    const fade = 0.020;

    button.addEventListener('click', () => {
      if (!context) {
        context = new AudioContext();
        gain = context.createGain();
        context.createMediaElementSource(audio).connect(gain);
        gain.connect(context.destination);
      }
      context.resume();

      let now = context.currentTime;
      gain.gain.cancelScheduledValues(now);

      if (audio.paused) {
        gain.gain.setValueAtTime(0, now);
        gain.gain.linearRampToValueAtTime(1, now + fade);
        audio.play();
      } else {
        gain.gain.setValueAtTime(gain.gain.value, now);
        gain.gain.linearRampToValueAtTime(0, now + fade);
        setTimeout(() => audio.pause(), fade * 1000);
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
