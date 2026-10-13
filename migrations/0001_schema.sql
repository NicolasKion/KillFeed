-- The killfeed schema. EVE ids (characters, killmails, types, systems,
-- corporations) are bigints handed out by the game, never by us; every
-- time is UTC.

-- Every character the site has seen: the ones an overlay was opened for
-- (with a name from ESI) and everyone on a stored killmail (id only).
-- `listening_since` is refreshed by every overlay poll; the listener
-- watches the characters refreshed in the last five minutes.
create table characters (
    id bigint primary key,
    name text,
    listening_since timestamptz,
    created_at timestamptz,
    updated_at timestamptz
);

create index characters_listening_since_index on characters (listening_since);

-- A killmail as ESI describes it (`data`, with every optional key
-- present), plus zKillboard's hash, value and solo flag and the ship's
-- meta group for the corner badge.
create table killmails (
    id bigint primary key,
    character_id bigint references characters (id),
    hash text not null,
    data jsonb not null,
    time timestamptz not null,
    solarsystem_id bigint not null,
    corporation_id bigint,
    alliance_id bigint,
    ship_type_id bigint not null,
    ship_meta_group_id smallint,
    damage_taken bigint not null,
    value double precision,
    solo boolean not null default false,
    created_at timestamptz,
    updated_at timestamptz
);

create index killmails_character_id_index on killmails (character_id);
create index killmails_solarsystem_id_index on killmails (solarsystem_id);
create index killmails_corporation_id_index on killmails (corporation_id);
create index killmails_alliance_id_index on killmails (alliance_id);
create index killmails_ship_type_id_index on killmails (ship_type_id);
-- The overlay's time window and the nightly purge.
create index killmails_time_index on killmails (time);

-- Who was on a killmail: the attackers with a character and a
-- corporation, and the victim.
create table killmail_involvements (
    id bigserial primary key,
    killmail_id bigint not null references killmails (id) on delete cascade,
    character_id bigint not null references characters (id) on delete cascade,
    is_attacker boolean not null default false,
    created_at timestamptz,
    updated_at timestamptz,
    unique (killmail_id, character_id)
);

create index killmail_involvements_character_id_index on killmail_involvements (character_id);

-- The meta group of a type from ESI's dogma attributes, kept for 30
-- days (static data only changes with game expansions). Failed lookups
-- are never stored.
create table ship_types (
    type_id bigint primary key,
    meta_group_id smallint not null,
    fetched_at timestamptz not null
);

-- zKillboard's listing of a character's latest killmails, kept for five
-- minutes so the overlay's 10 second poll stays within their limits.
create table zkillboard_listings (
    character_id bigint primary key,
    body jsonb not null,
    expires_at timestamptz not null
);

-- Where the firehose listener is: the last R2Z2 sequence it handled.
create table listener_state (
    name text primary key,
    sequence bigint not null,
    updated_at timestamptz not null
);

-- The durable job queue: processing a killmail and healing a missing
-- meta group. One pending job per kind and killmail (Laravel's
-- ShouldBeUnique); done jobs are deleted, failed ones kept for a week.
create table jobs (
    id bigserial primary key,
    kind text not null,
    killmail_id bigint not null,
    payload jsonb not null default '{}',
    attempts integer not null default 0,
    max_attempts integer not null default 1,
    available_at timestamptz not null,
    reserved_at timestamptz,
    failed_at timestamptz,
    last_error text,
    created_at timestamptz not null
);

create unique index jobs_pending_unique on jobs (kind, killmail_id) where failed_at is null;
create index jobs_due_index on jobs (available_at) where failed_at is null and reserved_at is null;
