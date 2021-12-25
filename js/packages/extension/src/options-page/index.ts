import { listenOption, Option, setOption } from '../options';

document.addEventListener('DOMContentLoaded', async () => {
  const legacyApiEl = document.getElementById('legacy-api') as HTMLInputElement;

  legacyApiEl.addEventListener('change', () => setOption(Option.UseLegacyApi, legacyApiEl.checked));
  listenOption<boolean | undefined>(Option.UseLegacyApi, v => {
    if (v === undefined) {
      legacyApiEl.indeterminate = true;
    } else if (v !== legacyApiEl.checked) {
      legacyApiEl.checked = v;
    }
  });
});
