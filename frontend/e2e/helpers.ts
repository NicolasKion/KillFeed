// Shared helpers for the e2e suite: everything talks to the docker stack,
// so the database is seeded and inspected through psql in the
// killfeed-postgres container, the way the Pest browser tests used the
// factories. The stack runs with EVE_APIS_ENABLED=false: zKillboard and
// ESI calls fail at once, so a character the page needs is seeded with
// its name (what the Pest tests' ESI fake answered).
import { execFileSync } from 'node:child_process';
import { readFileSync } from 'node:fs';
import { expect, type Page } from '@playwright/test';

export function psql(sql: string): string {
  return execFileSync(
    'docker',
    ['exec', 'killfeed-postgres', 'psql', '-U', 'killfeed', '-d', 'killfeed', '-qtAc', sql],
    { encoding: 'utf8' },
  ).trim();
}

/** A SQL string literal. */
export function quote(value: string | null): string {
  return value === null ? 'null' : `'${value.replaceAll("'", "''")}'`;
}

/** Laravel's RefreshDatabase: every table empty. */
export function resetDatabase(): void {
  psql(
    `truncate characters, killmails, killmail_involvements, ship_types, zkillboard_listings,
       listener_state, jobs restart identity cascade`,
  );
}

let sequence = 0;

/** `Character::factory()->create()`. */
export function createCharacter(attributes: { id?: number; name?: string } = {}): number {
  sequence += 1;
  const id = attributes.id ?? 90_500_000 + sequence;
  psql(`insert into characters (id, name, created_at, updated_at)
        values (${id}, ${quote(attributes.name ?? `Test Pilot ${sequence}`)}, now(), now())`);
  return id;
}

/** `Killmail::factory()->create()`: a killmail of the last hour. */
export function createKillmail(
  attributes: { value?: number | null; metaGroup?: number | null; characterId?: number } = {},
): number {
  sequence += 1;
  const id = 130_000_000 + sequence;
  const value = attributes.value === undefined ? 1_000_000 : attributes.value;
  const metaGroup = attributes.metaGroup === undefined ? 1 : attributes.metaGroup;
  psql(`insert into killmails (id, character_id, hash, data, time, solarsystem_id, corporation_id,
          ship_type_id, ship_meta_group_id, damage_taken, value, solo, created_at, updated_at)
        values (${id}, ${attributes.characterId ?? 'null'}, 'hash${id}', '[]',
          now() - interval '${sequence} minutes', 30002187, 98000001, 587,
          ${metaGroup ?? 'null'}, 5000, ${value ?? 'null'}, false, now(), now())`);
  return id;
}

/** `$killmail->killmailInvolvements()->create([...])`. */
export function involve(killmailId: number, characterId: number, isAttacker: boolean): void {
  psql(`insert into killmail_involvements (killmail_id, character_id, is_attacker, created_at, updated_at)
        values (${killmailId}, ${characterId}, ${isAttacker}, now(), now())`);
}

export function baseUrl(): string {
  return process.env.E2E_BASE_URL ?? 'http://localhost:5316';
}

/** Collects uncaught page errors and console errors, Pest's
 * assertNoJavascriptErrors. The ship renders and portraits come from
 * images.evetech.net; a missing render is not an error of the page. */
export function trackJavascriptErrors(page: Page): () => void {
  const errors: string[] = [];
  page.on('pageerror', (error) => errors.push(error.message));
  page.on('console', (message) => {
    if (message.type() === 'error' && !message.location().url.includes('images.evetech.net')) {
      errors.push(message.text());
    }
  });
  return () => expect(errors, 'javascript errors').toEqual([]);
}

/** Pest's `assertSee`: the text is on the page. */
export async function assertSee(page: Page, text: string): Promise<void> {
  await expect(page.locator('body')).toContainText(text);
}

/** A gray square for every image of images.evetech.net, so screenshots
 * and tests never depend on the image server. */
export async function stubEveImages(page: Page): Promise<void> {
  const png = Buffer.from(
    'iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNkaGD4DwAD/wH+0uK0eAAAAABJRU5ErkJggg==',
    'base64',
  );
  await page.route('https://images.evetech.net/**', (route) =>
    route.fulfill({ status: 200, contentType: 'image/png', body: png }),
  );
}

/** Math.random as a seeded generator (mulberry32), so the simulated feed
 * draws the same killmails on every run and on both stacks. */
export async function seedRandom(page: Page, seed = 42): Promise<void> {
  await page.addInitScript((initial) => {
    let state = initial;
    Math.random = () => {
      state = (state + 0x6d2b79f5) | 0;
      let t = Math.imul(state ^ (state >>> 15), 1 | state);
      t = (t + Math.imul(t ^ (t >>> 7), 61 | t)) ^ t;
      return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
    };
  }, seed);
}

type DatasetRow = Record<string, unknown>;

/** tests/fixtures/dataset.json, the rows the Laravel fixtures were
 * recorded with, seeded into the stack relative to now (the visual
 * comparison and the SEO heads). */
export function seedDataset(): void {
  const path = new URL('../../tests/fixtures/dataset.json', import.meta.url);
  const dataset = JSON.parse(readFileSync(path, 'utf8')) as Record<string, DatasetRow[]>;
  const now = Date.now() - (Date.now() % 1000);
  const resolve = (value: unknown): unknown => {
    if (value && typeof value === 'object' && !Array.isArray(value)) {
      const fields = value as Record<string, unknown>;
      if ('offset_minutes' in fields) {
        const moment = new Date(now + (fields.offset_minutes as number) * 60_000).toISOString();
        return fields.esi
          ? `${moment.slice(0, 19)}Z`
          : `${moment.slice(0, 19).replace('T', ' ')}+00`;
      }
      return Object.fromEntries(Object.entries(fields).map(([k, v]) => [k, resolve(v)]));
    }
    if (Array.isArray(value)) {
      return value.map(resolve);
    }
    return value;
  };
  resetDatabase();
  for (const table of ['characters', 'killmails', 'killmail_involvements']) {
    for (const row of dataset[table]) {
      psql(
        `insert into ${table} select * from jsonb_populate_record(null::${table}, ${quote(JSON.stringify(resolve(row)))}::jsonb)`,
      );
    }
  }
  psql(
    `select setval(pg_get_serial_sequence('killmail_involvements', 'id'), (select max(id) from killmail_involvements))`,
  );
  // The meta groups the reference's ESI fake answered for the demo ships.
  const fakes = JSON.parse(
    readFileSync(new URL('../../tests/fixtures/eve.json', import.meta.url), 'utf8'),
  ) as Record<
    string,
    { body: { type_id?: number; dogma_attributes?: { attribute_id: number; value: number }[] } }
  >;
  for (const [key, answer] of Object.entries(fakes)) {
    if (!key.startsWith('esi:universe/types/')) {
      continue;
    }
    const attributes = new Map(
      (answer.body.dogma_attributes ?? []).map((attribute) => [
        attribute.attribute_id,
        attribute.value,
      ]),
    );
    const tech = attributes.get(422) ?? 1;
    const metaGroup = attributes.get(1692) ?? (tech === 2 ? 2 : tech === 3 ? 14 : 1);
    psql(`insert into ship_types (type_id, meta_group_id, fetched_at)
          values (${answer.body.type_id}, ${metaGroup}, now())`);
  }
}
