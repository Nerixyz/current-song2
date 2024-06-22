import { html, LitElement, PropertyValues } from 'lit';
import { customElement, property, state } from 'lit/decorators';
import { listenOption, Option, setOption } from '../options';

@customElement('text-option')
export class TextOption extends LitElement {
  @property({ type: String })
  option!: Option;
  @property({ type: String, attribute: 'default' })
  defaultValue = '';

  private _prevListener?: () => void;

  @state()
  private _value?: string;

  protected updated(changedProperties: PropertyValues) {
    super.updated(changedProperties);
    if (!this._prevListener || changedProperties.has('option')) {
      this._prevListener?.();
      this._prevListener = listenOption<string>(this.option, value => {
        this._value = value;
        this.requestUpdate('_value');
      });
    }
  }

  protected render() {
    return html`
      <input type="text" @input="${this._updateHtmlValue}" value=${this._value} placeholder="${this.defaultValue}" />
    `;
  }

  private _updateHtmlValue(e: Event) {
    const target = e.target as HTMLInputElement;
    this._value = target.value || undefined;
    this._trySetValue();
    this.requestUpdate();
  }

  private _trySetValue() {
    setOption(this.option, this._value).catch(console.error);
  }
}
