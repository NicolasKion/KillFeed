import type { OverlayCharacter, OverlayKillmail, SimShip } from '#lib/killfeed.ts';

export const SIMULATED_CHARACTER: OverlayCharacter = { id: 1, name: 'Simulated Capsuleer' };

export type SimulatedKillfeed = {
  character: OverlayCharacter;
  readonly killmails: OverlayKillmail[];
  readonly killsValue: number;
  readonly lossesValue: number;
  readonly killmailsCount: number;
  seed(count: number): void;
  start(): void;
  stop(): void;
};

export function createSimulatedKillfeed(_options: {
  ships: SimShip[];
  maxKillmails?: number;
  random?: () => number;
}): SimulatedKillfeed {
  throw new Error('not implemented');
}
