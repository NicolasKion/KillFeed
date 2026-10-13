import { defineParams } from '@sveltejs/kit/params';

export const params = defineParams({
  // `whereNumber`: the overlay of a character is any digit string,
  // leading zeros included.
  numeric: (param) => (/^\d+$/.test(param) ? param : undefined),
});
