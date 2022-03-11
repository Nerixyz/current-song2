import { splitTitle } from '../../src/utils/text';

describe('splitTitle', function () {
  it('should work', function () {
    const inputs = [
      'Forsen - My Cool Video',
      'Without Dash',
      'AlienDancing by Obama',
      'LULE - AlienGathering (forsen remix)',
      'Oops - My chair broke [xd] (feat. alien)',
      'Here - Are - Multiple - Dashes',
      'Forsen - Aliens (Forsen-Remix)', // check if this is really expected
    ].map(title => splitTitle(title));

    expect(inputs).toMatchSnapshot();
  });
});
