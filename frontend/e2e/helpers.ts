// Shared helpers for the e2e suite: everything talks to the docker stack,
// so the database is seeded and inspected through psql in the
// killfeed-postgres container. The stack runs with EVE_APIS_ENABLED=false:
// zKillboard and ESI calls fail at once, so a character the page needs is
// seeded with its name.
import { execFileSync } from 'node:child_process';
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

/** Every table empty. */
export function resetDatabase(): void {
  psql(
    `truncate characters, killmails, killmail_involvements, ship_types, zkillboard_listings,
       listener_state, jobs restart identity cascade`,
  );
}

let sequence = 0;

/** A character with a name. */
export function createCharacter(attributes: { id?: number; name?: string } = {}): number {
  sequence += 1;
  const id = attributes.id ?? 90_500_000 + sequence;
  psql(`insert into characters (id, name, created_at, updated_at)
        values (${id}, ${quote(attributes.name ?? `Test Pilot ${sequence}`)}, now(), now())`);
  return id;
}

/** A killmail of the last hour. */
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

/** A character on a killmail, as attacker or victim. */
export function involve(killmailId: number, characterId: number, isAttacker: boolean): void {
  psql(`insert into killmail_involvements (killmail_id, character_id, is_attacker, created_at, updated_at)
        values (${killmailId}, ${characterId}, ${isAttacker}, now(), now())`);
}

export function baseUrl(): string {
  return process.env.E2E_BASE_URL ?? 'http://localhost:5316';
}

/** Collects uncaught page errors and console errors. The ship renders and portraits come from
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

/** The text is on the page. */
export async function assertSee(page: Page, text: string): Promise<void> {
  await expect(page.locator('body')).toContainText(text);
}

/** A gray square for every image of images.evetech.net, so the tests
 * never depend on the image server. */
export async function stubEveImages(page: Page): Promise<void> {
  const png = Buffer.from(
    'iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNkaGD4DwAD/wH+0uK0eAAAAABJRU5ErkJggg==',
    'base64',
  );
  await page.route('https://images.evetech.net/**', (route) =>
    route.fulfill({ status: 200, contentType: 'image/png', body: png }),
  );
}
