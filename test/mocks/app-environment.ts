// Vitest stand-in for SvelteKit's `$app/environment`.
// Tests run in jsdom; we need `browser` to be addressable but default off so
// stores don't auto-load from a real localStorage in their constructors.

export const browser = false;
export const dev = false;
export const building = false;
export const version = "test";
