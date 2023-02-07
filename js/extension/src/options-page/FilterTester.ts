import { css, html, LitElement } from 'lit';
import { customElement, state } from 'lit/decorators';
import { FilterManager } from '../filters/FilterManager';
import { LocalFilterStorage } from '../filters/FilterStorage';

@customElement('filter-tester')
export class FilterTester extends LitElement {
  static styles = css`
    .will-send {
      text-decoration: underline green 3px;
    }
    .wont-send {
      text-decoration: underline red 3px;
    }
  `;

  private _filterManager = new FilterManager(LocalFilterStorage, () => this._onFilterUpdate());
  @state()
  private _urlWillBeSent = true;

  protected render() {
    return html`
      <div>
        <label>
          Test a URL here
          <input id="url-input" type="text" placeholder="URL" @input=${this._onUrlChange} />
        </label>
        ${this._urlWillBeSent
          ? html`<div class="will-send">Media info from this URL will be sent.</div>`
          : html`<div class="wont-send">Media info from this URL won't be sent.</div>`}
      </div>
    `;
  }

  private _onUrlChange(e: Event) {
    this._urlWillBeSent = this._filterManager.checkUrl((e.target as HTMLInputElement).value);
    this.requestUpdate('_urlWillBeSent');
  }

  private _onFilterUpdate() {
    const el = this.renderRoot.querySelector('#url-input') as HTMLInputElement;
    this._urlWillBeSent = this._filterManager.checkUrl(el.value);
    this.requestUpdate('_urlWillBeSent');
  }
}
