import type { OverlayCharacter, OverlayKillmail, SimShip } from '@/lib/killfeed';
import { onUnmounted, ref } from 'vue';

const SIMULATED_CHARACTER: OverlayCharacter = {
    id: 1,
    name: 'Simulated Capsuleer',
};

const randomBetween = (min: number, max: number) => min + Math.random() * (max - min);

const pick = <T>(items: T[]): T => items[Math.floor(Math.random() * items.length)];

export function useSimulatedKillfeed(options: { ships: SimShip[]; maxKillmails?: number }) {
    const maxKillmails = Math.max(1, options.maxKillmails ?? 50);
    const ships = options.ships;

    const killmails = ref<OverlayKillmail[]>([]);
    const killsValue = ref(0);
    const lossesValue = ref(0);
    const killmailsCount = ref(0);

    let nextId = 1;
    let timer: number | undefined;

    const makeKillmail = (): OverlayKillmail => {
        const ship = pick(ships);
        const isLoss = Math.random() < 0.3;
        const solo = Math.random() < 0.12;
        // Log-uniform between ~1.5M and ~3B ISK, with the occasional whale.
        const value = Math.round(10 ** randomBetween(6.2, Math.random() < 0.04 ? 10.4 : 9.5));
        const involved = solo ? 1 : Math.ceil(randomBetween(1, 40) ** 2 / 40);

        return {
            id: nextId++,
            character_id: isLoss ? SIMULATED_CHARACTER.id : null,
            ship_type_id: ship.type_id,
            ship_meta_group_id: ship.meta_group_id,
            value,
            solo,
            killmail_involvements: Array.from({ length: involved }, () => ({ character_id: null })),
        };
    };

    const push = () => {
        const killmail = makeKillmail();

        killmails.value = [killmail, ...killmails.value].slice(0, maxKillmails);
        killmailsCount.value += 1;

        if (killmail.character_id === SIMULATED_CHARACTER.id) {
            lossesValue.value += killmail.value;
        } else {
            killsValue.value += killmail.value;
        }
    };

    const seed = (count: number) => {
        for (let i = 0; i < count; i++) {
            push();
        }
    };

    const scheduleNext = () => {
        timer = window.setTimeout(
            () => {
                push();
                scheduleNext();
            },
            randomBetween(2500, 6000),
        );
    };

    const start = () => {
        if (timer === undefined) {
            scheduleNext();
        }
    };

    const stop = () => {
        window.clearTimeout(timer);
        timer = undefined;
    };

    onUnmounted(stop);

    return {
        character: SIMULATED_CHARACTER,
        killmails,
        killsValue,
        lossesValue,
        killmailsCount,
        seed,
        start,
        stop,
    };
}
