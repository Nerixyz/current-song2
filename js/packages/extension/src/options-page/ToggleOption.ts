import { html, LitElement, PropertyDeclaration, PropertyValues } from 'lit';
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
      console.log('init');
      this._prevListener?.();
      console.log(this.option);
      this._prevListener = listenOption<boolean>(this.option, value => {
        this._value = value;
        this.requestUpdate('_value');
      });
    }
  }

  protected render() {
    console.log('render', this._value);
    return html`<label>
      ${this.label}
      <input type="checkbox" ?checked=${this._value} @change=${this._changed} />
    </label>`;
  }

  private _changed(e: Event) {
    console.log('changed');
    this._value = (e.target as HTMLInputElement).checked;
    setOption(this.option, this._value).catch(console.error);
  }
}
