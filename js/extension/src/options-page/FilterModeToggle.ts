import { css, html, LitElement } from 'lit';
import { customElement, state } from 'lit/decorators';
import { classMap } from 'lit/directives/class-map';
import { DEFAULT_FILTER_MODE, FilterMode, listenOption, Option, setOption } from '../options';

@customElement('filter-mode-toggle')
export class FilterModeToggle extends LitElement {
  static styles = css`
    button {
      border-radius: 0.25rem;
      border: none;
      background: #323232;
      color: white;
      cursor: pointer;
      padding: 0.5rem 1rem;
      margin: 0 0.5rem;
    }

    button:hover:not(.active) {
      outline: solid white 1px;
    }

    .active {
      outline: solid red 2px;
    }
  `;

  @state()
  private _mode = DEFAULT_FILTER_MODE;

  constructor() {
    super();
    listenOption<FilterMode>(Option.FilterMode, v => {
      this._mode = v ?? this._mode;
      this.requestUpdate('_mode');
    });
  }

  protected render(): unknown {
    return html`<button class=${classMap({ active: this._mode === FilterMode.Allow })} @click=${this._onAllow}>
        Allow</button
      ><button class=${classMap({ active: this._mode === FilterMode.Block })} @click=${this._onBlock}>Block</button>`;
  }

  private _onAllow() {
    setOption(Option.FilterMode, FilterMode.Allow).catch(console.error);
  }
  private _onBlock() {
    setOption(Option.FilterMode, FilterMode.Block).catch(console.error);
  }
}
