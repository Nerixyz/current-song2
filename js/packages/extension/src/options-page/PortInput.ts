import { css, html, LitElement } from 'lit';
import { customElement, state } from 'lit/decorators';
import { DEFAULT_CURRENT_PORT, DEFAULT_LEGACY_PORT, listenOption, Option, setOption } from '../options';

const PORT_REGEX = /^\d{2,6}$/;

@customElement('port-input')
export class PortInput extends LitElement {
  static styles = css`
    .invalid {
      outline: red solid 2px;
    }
  `;

  @state()
  private _port: string | undefined | null = undefined;
  @state()
  private _useLegacyApi = false;

  constructor() {
    super();
    listenOption<string>(Option.ApiPort, v => {
      const shouldUpdate = this._port !== v;
      this._port = v;
      if (shouldUpdate) this.requestUpdate();
    });
    listenOption<boolean>(Option.UseLegacyApi, v => {
      this._useLegacyApi = v ?? false;
      this.requestUpdate();
    });
  }

  protected render() {
    const defaultPort = this._getCurrentDefaultPort();
    const port = this._getActualPort();
    const isValid = PORT_REGEX.test(port);
    return html`
      <input
        type="text"
        aria-invalid=${!isValid}
        class=${isValid ? 'valid' : 'invalid'}
        @input="${this._updatePort}"
        value=${port === defaultPort ? undefined : port}
        placeholder=${this._getCurrentDefaultPort()}
      />
    `;
  }

  private _updatePort(e: Event) {
    const target = e.target as HTMLInputElement;
    if (!this._port && target.value === this._getCurrentDefaultPort()) {
      return;
    }
    this._port = target.value || null;
    this._trySetPort();
    this.requestUpdate();
  }

  private _getCurrentDefaultPort() {
    return (this._useLegacyApi ? DEFAULT_LEGACY_PORT : DEFAULT_CURRENT_PORT).toString();
  }

  private _getActualPort() {
    return this._port ?? this._getCurrentDefaultPort();
  }

  private _trySetPort() {
    if (this._port) {
      if (PORT_REGEX.test(this._port)) {
        setOption(Option.ApiPort, Number(this._port)).catch(console.error);
      }
    } else {
      setOption(Option.ApiPort, null).catch(console.error);
    }
  }
}
