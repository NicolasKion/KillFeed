// The simulated killfeed of the setup page and the demo: synthetic
// killmails of the demo ships, a new one every 2.5 to 6 seconds. The
// draws from `random` happen in the order the Vue version made them, so
// a seeded Math.random gives the same feed on both.
import type { OverlayCharacter, OverlayKillmail, SimShip } from '#lib/killfeed.ts';

export const SIMULATED_CHARACTER: OverlayCharacter = {
  id: 1,
  name: 'Simulated Capsuleer',
};

class SimulatedKillfeed {
  character = SIMULATED_CHARACTER;
  killmails = $state<OverlayKillmail[]>([]);
  killsValue = $state(0);
  lossesValue = $state(0);
  killmailsCount = $state(0);

  #ships: SimShip[];
  #maxKillmails: number;
  #random: () => number;
  #nextId = 1;
  #timer: ReturnType<typeof setTimeout> | undefined;

  constructor(options: { ships: SimShip[]; maxKillmails?: number; random?: () => number }) {
    this.#ships = options.ships;
    this.#maxKillmails = Math.max(1, options.maxKillmails ?? 50);
    this.#random = options.random ?? Math.random;
  }

  #between(min: number, max: number): number {
    return min + this.#random() * (max - min);
  }

  #makeKillmail(): OverlayKillmail {
    const ship = this.#ships[Math.floor(this.#random() * this.#ships.length)];
    const isLoss = this.#random() < 0.3;
    const solo = this.#random() < 0.12;
    // Log-uniform between ~1.5M and ~3B ISK, with the occasional whale.
    const ceiling = this.#random() < 0.04 ? 10.4 : 9.5;
    const value = Math.round(10 ** this.#between(6.2, ceiling));
    const involved = solo ? 1 : Math.ceil(this.#between(1, 40) ** 2 / 40);

    return {
      id: this.#nextId++,
      character_id: isLoss ? SIMULATED_CHARACTER.id : null,
      ship_type_id: ship.type_id,
      ship_meta_group_id: ship.meta_group_id,
      value,
      solo,
      killmail_involvements: Array.from({ length: involved }, () => ({ character_id: null })),
    };
  }

  #push(): void {
    const killmail = this.#makeKillmail();
    this.killmails = [killmail, ...this.killmails].slice(0, this.#maxKillmails);
    this.killmailsCount += 1;
    if (killmail.character_id === SIMULATED_CHARACTER.id) {
      this.lossesValue += killmail.value!;
    } else {
      this.killsValue += killmail.value!;
    }
  }

  seed(count: number): void {
    for (let i = 0; i < count; i++) {
      this.#push();
    }
  }

  #scheduleNext(): void {
    this.#timer = setTimeout(
      () => {
        this.#push();
        this.#scheduleNext();
      },
      this.#between(2500, 6000),
    );
  }

  start(): void {
    if (this.#timer === undefined) {
      this.#scheduleNext();
    }
  }

  stop(): void {
    clearTimeout(this.#timer);
    this.#timer = undefined;
  }
}

export type { SimulatedKillfeed };

export function createSimulatedKillfeed(options: {
  ships: SimShip[];
  maxKillmails?: number;
  random?: () => number;
}): SimulatedKillfeed {
  return new SimulatedKillfeed(options);
}
