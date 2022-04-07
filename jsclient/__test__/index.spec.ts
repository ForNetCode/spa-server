import test from 'ava'

import { forTest } from '../index'

test('sync function from native code', (t) => {
  t.is(forTest(2), 34)
})
