import { html, LitElement } from 'lit';
import { FilterModel } from '../filters/types';
import { customElement, query, state } from 'lit/decorators';
import { listenJsonOption, Option, setJsonOption } from '../options';

@customElement('filter-list')
export class FilterList extends LitElement {
  @state()
  private _filters: FilterModel[] = [];

  // This should be @query('..') but somehow this doesn't work.
  private get _filterInput(): HTMLInputElement {
    return this.renderRoot.querySelector('#filter-text') as HTMLInputElement;
  }
  private get _filterIsRegex(): HTMLInputElement {
    return this.renderRoot.querySelector('#filter-is-regex') as HTMLInputElement;
  }

  constructor() {
    super();
    listenJsonOption<FilterModel[]>(Option.Filters, value => {
      this._filters = value ?? this._filters;
      this.requestUpdate('_filters');
    });
  }

  protected render() {
    return html`
      <form @submit=${this._addFilter}>
        <input id="filter-text" type="text" placeholder="Filter" />
        <label>Regex<input id="filter-is-regex" type="checkbox" /></label>
        <input type="submit" value="Add" />
      </form>
      <div>
        <ul>
          ${this._filters.map(
            f =>
              html`<li>
                ${f.isRegex ? '[Regex]' : ''} <span class="code">${f.value}</span>
                <button @click=${() => this._removeFilter(f)}>Remove</button>
              </li>`,
          )}
        </ul>
      </div>
    `;
  }

  private _addFilter(e: Event) {
    e.preventDefault();
    if (this._filterInput.value.length > 1) {
      const isRegex = this._filterIsRegex.checked;
      const value = this._filterInput.value;
      if (isRegex) {
        try {
          new RegExp(value);
        } catch (e) {
          console.warn(e);
          return;
        }
      }
      setJsonOption(Option.Filters, [...this._filters, { isRegex, value } as FilterModel]).catch(console.error);
      this._filterInput.value = '';
    }
  }

  private _removeFilter(filter: FilterModel) {
    setJsonOption(
      Option.Filters,
      this._filters.filter(f => f !== filter),
    ).catch(console.error);
  }
}
