import { html, LitElement, PropertyValues } from 'lit';
import { customElement, property, state } from 'lit/decorators';
import { listenOption, Option, setOption } from '../options';

@customElement('toggle-option')
export class ToggleOption extends LitElement {
  @property({ type: String })
  option!: Option;
  @property({ type: String })
  label = '';

  private _prevListener?: () => void;

  @state()
  private _value?: boolean;

  protected updated(changedProperties: PropertyValues) {
    super.updated(changedProperties);
    if (!this._prevListener || changedProperties.has('option')) {
      this._prevListener?.();
      this._prevListener = listenOption<boolean>(this.option, value => {
        this._value = value;
        this.requestUpdate('_value');
      });
    }
  }

  protected render() {
    return html`<label>
      ${this.label}
      <input type="checkbox" ?checked=${this._value} @change=${this._changed} />
    </label>`;
  }

  private _changed(e: Event) {
    this._value = (e.target as HTMLInputElement).checked;
    setOption(this.option, this._value).catch(console.error);
  }
}
