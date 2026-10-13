#!/usr/bin/env bash
# Runs the Laravel reference (the `main` branch) on http://127.0.0.1:8316
# for recording golden fixtures (scripts/laravel-reference/record.py).
#
#   scripts/laravel-reference/run.sh setup        export main, install, build assets
#   scripts/laravel-reference/run.sh start        fresh database, serve
#   scripts/laravel-reference/run.sh artisan ...  run artisan in the reference container
#   scripts/laravel-reference/run.sh php '...'    run PHP code with the app booted (tinker)
#   scripts/laravel-reference/run.sh sql "..."    run SQL against the reference database
#   scripts/laravel-reference/run.sh stop
#
# The checkout lives in $LARAVEL_REF_DIR (default /tmp/killfeed-laravel-ref),
# never inside this repository. MySQL 8 like production, the database
# queue and cache like production, and HttpFakesServiceProvider.php
# answering every outgoing HTTP call (no ESI, zKillboard or R2Z2).
# Containers are named killfeed-laravel-*, on the killfeed-laravel network.
set -euo pipefail

repo=$(cd "$(dirname "$0")/../.." && pwd)
dir=${LARAVEL_REF_DIR:-/tmp/killfeed-laravel-ref}
image=killfeed-laravel-ref
network=killfeed-laravel
port=8316

run() {
  docker run --rm -u "$(id -u):$(id -g)" -e HOME=/tmp -e COMPOSER_HOME=/tmp/composer \
    --network "$network" -v "$dir/app:/app" "$@"
}

mysql_up() {
  docker network inspect "$network" >/dev/null 2>&1 || docker network create "$network" >/dev/null
  if ! docker inspect killfeed-laravel-mysql >/dev/null 2>&1; then
    docker run -d --name killfeed-laravel-mysql --network "$network" \
      -e MYSQL_ROOT_PASSWORD=killfeed -e MYSQL_DATABASE=killfeed \
      mysql:8.4 --character-set-server=utf8mb4 --collation-server=utf8mb4_unicode_ci >/dev/null
  fi
  until docker exec killfeed-laravel-mysql mysql -uroot -pkillfeed -e 'select 1' >/dev/null 2>&1; do sleep 1; done
}

case "${1:-}" in
  setup)
    docker build -t "$image" "$repo/scripts/laravel-reference"
    rm -rf "$dir" && mkdir -p "$dir/app"
    git -C "$repo" archive main | tar -x -C "$dir/app"
    cp "$repo/scripts/laravel-reference/env" "$dir/app/.env"
    cp "$repo/scripts/laravel-reference/HttpFakesServiceProvider.php" "$dir/app/app/Providers/"
    sed -i 's/App\\Providers\\AppServiceProvider::class,/&\n    App\\Providers\\HttpFakesServiceProvider::class,/' "$dir/app/bootstrap/providers.php"
    mysql_up
    run --name killfeed-laravel-setup "$image" sh -c '
      composer install --no-interaction --no-progress
      npm ci --no-audit --no-fund
      npx vite build'
    ;;
  start)
    mysql_up
    rm -f "$dir/app/storage/logs/laravel.log" "$dir/app/storage/logs/http.jsonl"
    docker exec killfeed-laravel-mysql mysql -uroot -pkillfeed \
      -e 'drop database if exists killfeed; create database killfeed' 2>/dev/null
    run --name killfeed-laravel-migrate "$image" php artisan migrate --force >/dev/null
    docker rm -f killfeed-laravel-ref >/dev/null 2>&1 || true
    docker run -d --name killfeed-laravel-ref -u "$(id -u):$(id -g)" -e HOME=/tmp \
      -e PHP_CLI_SERVER_WORKERS=4 --network "$network" \
      -v "$dir/app:/app" \
      -p "127.0.0.1:$port:$port" "$image" \
      php artisan serve --host=0.0.0.0 --port="$port" --no-reload >/dev/null
    until curl -fs "http://127.0.0.1:$port/up" >/dev/null; do sleep 0.5; done
    echo "Laravel reference on http://127.0.0.1:$port"
    ;;
  artisan)
    shift
    docker exec killfeed-laravel-ref php artisan "$@"
    ;;
  php)
    shift
    docker exec -i killfeed-laravel-ref php artisan tinker --execute="$*"
    ;;
  sql)
    shift
    docker exec -i killfeed-laravel-mysql mysql -uroot -pkillfeed --default-character-set=utf8mb4 -N -B -r killfeed -e "$*" 2>/dev/null
    ;;
  stop)
    docker rm -f killfeed-laravel-ref killfeed-laravel-mysql >/dev/null 2>&1 || true
    docker network rm "$network" >/dev/null 2>&1 || true
    ;;
  *)
    sed -n '2,16p' "$0"
    exit 1
    ;;
esac
