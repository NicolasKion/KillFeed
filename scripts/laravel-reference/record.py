#!/usr/bin/env python3
"""Records golden fixtures from the running Laravel reference.

    scripts/laravel-reference/run.sh start
    python3 -I scripts/laravel-reference/record.py
    python3 -I scripts/laravel-reference/record.py seed   (the dataset only)

Every scenario starts from tests/fixtures/dataset.json (times relative to
the moment of the recording) with the outgoing HTTP calls answered from
tests/fixtures/eve.json, the same answers the Rust suites' mock servers
give. Writes tests/fixtures/golden/:

- pages/<name>.json: one GET of an Inertia page (status, and the page
  object from the HTML), plus the rows it left behind: the characters and
  the queued jobs.
- http/<name>.json (+ .html): the answers that are not Inertia pages (the
  health page, the preview JSON, error pages, the dropped package routes'
  status, the 409 a stale Inertia client gets).
- jobs.json: the overlay page queuing the zKillboard backfill and the meta
  group heal, then the queue worker running them: the jobs, the stored
  killmails and characters, the failed jobs.
- meta.json: the moment of the recording, so the Rust suites can pin
  their clock to it.

Only the standard library; run with `python3 -I`.
"""

import datetime
import json
import os
import re
import subprocess
import sys
import urllib.error
import urllib.request

BASE = "http://127.0.0.1:8316"
APP_URL = "https://killfeed.killmail.space"
REPO = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
REF_DIR = os.environ.get("LARAVEL_REF_DIR", "/tmp/killfeed-laravel-ref")
APP = os.path.join(REF_DIR, "app")
OUT = os.path.join(REPO, "tests/fixtures/golden")
RUN = os.path.join(REPO, "scripts/laravel-reference/run.sh")
DATASET = json.load(open(os.path.join(REPO, "tests/fixtures/dataset.json"), encoding="utf-8"))
FAKES = json.load(open(os.path.join(REPO, "tests/fixtures/eve.json"), encoding="utf-8"))
NOW = datetime.datetime.now(datetime.timezone.utc).replace(microsecond=0)
TABLES = ["killmail_involvements", "killmails", "characters", "cache", "cache_locks", "jobs", "failed_jobs"]


class NoRedirect(urllib.request.HTTPRedirectHandler):
    def redirect_request(self, *args, **kwargs):
        return None


OPENER = urllib.request.build_opener(NoRedirect)


def request(path, headers=None):
    req = urllib.request.Request(BASE + path, headers=headers or {})
    try:
        with OPENER.open(req) as response:
            return response.status, dict(response.headers), response.read()
    except urllib.error.HTTPError as error:
        return error.code, dict(error.headers), error.read()


def resolve(value):
    """Dataset times: `{offset_minutes}` from NOW, as MySQL `datetime`, or
    with `esi` as ESI writes `killmail_time`."""
    if isinstance(value, dict) and "offset_minutes" in value:
        moment = NOW + datetime.timedelta(minutes=value["offset_minutes"])
        if value.get("esi"):
            return moment.strftime("%Y-%m-%dT%H:%M:%SZ")
        return moment.strftime("%Y-%m-%d %H:%M:%S")
    if isinstance(value, dict):
        return {key: resolve(item) for key, item in value.items()}
    if isinstance(value, list):
        return [resolve(item) for item in value]
    return value


def sql(statement):
    return subprocess.run([RUN, "sql", statement], check=True, capture_output=True, text=True).stdout


def php(code):
    out = subprocess.run([RUN, "php", code], check=True, capture_output=True, text=True).stdout
    return out.strip()


def literal(value):
    if value is None:
        return "null"
    if isinstance(value, bool):
        return "1" if value else "0"
    if isinstance(value, (int, float)):
        return repr(value)
    if isinstance(value, (dict, list)):
        value = json.dumps(value)
    return "'" + str(value).replace("\\", "\\\\").replace("'", "\\'") + "'"


def seed():
    statements = ["set foreign_key_checks = 0"]
    statements += [f"truncate table {table}" for table in TABLES]
    statements.append("set foreign_key_checks = 1")
    for table in ["characters", "killmails", "killmail_involvements"]:
        for row in DATASET[table]:
            row = resolve(row)
            columns = ", ".join(row)
            values = ", ".join(literal(value) for value in row.values())
            statements.append(f"insert into {table} ({columns}) values ({values})")
    sql(";\n".join(statements))


def write_fakes():
    """eve.json as the URLs the Laravel app calls."""
    urls = {}
    for key, answer in FAKES.items():
        service, path = key.split(":", 1)
        if service == "esi":
            url = f"https://esi.evetech.net/latest/{path}/"
        else:
            url = f"https://zkillboard.com/{path}"
        urls[url] = resolve(answer)
    with open(os.path.join(APP, "storage/app/http-fakes.json"), "w", encoding="utf-8") as file:
        json.dump(urls, file)
    log = os.path.join(APP, "storage/logs/http.jsonl")
    if os.path.exists(log):
        os.remove(log)


def http_log():
    """The outgoing requests of the scenario, without Inertia's SSR call."""
    log = os.path.join(APP, "storage/logs/http.jsonl")
    if not os.path.exists(log):
        return []
    entries = [json.loads(line) for line in open(log, encoding="utf-8")]
    return [entry["url"] for entry in entries if ":13714" not in entry["url"]]


def characters():
    return json.loads(php("echo App\\Models\\Character::query()->orderBy('id')->get()->toJson();"))


def killmails():
    return json.loads(php(
        "echo App\\Models\\Killmail::query()->with(['killmailInvolvements' => fn ($q) => $q->orderBy('id')])"
        "->orderBy('id')->get()->toJson();"))


def jobs():
    rows = sql("select payload from jobs order by id").splitlines()
    out = []
    for row in rows:
        payload = json.loads(row)
        command = payload["data"]["command"]
        out.append({
            "job": payload["displayName"].rsplit("\\", 1)[-1],
            "killmail_id": int(re.search(r'killmail_id";i:(\d+)', command).group(1)),
            "with_zkill": 'zKillKillmail";O:' in command,
            "with_esi": 'esiKillmail";O:' in command,
        })
    return out


def failed_jobs():
    rows = sql("select payload, substring_index(exception, '\\n', 1) from failed_jobs order by id").splitlines()
    out = []
    for row in rows:
        payload, exception = row.split("\t", 1)
        payload = json.loads(payload)
        out.append({
            "job": payload["displayName"].rsplit("\\", 1)[-1],
            "killmail_id": int(re.search(r'killmail_id";i:(\d+)', payload["data"]["command"]).group(1)),
            "exception": exception.split(":", 1)[0],
        })
    return out


def page_object(body):
    match = re.search(rb'<script data-page="app" type="application/json">(.*?)</script>', body, re.S)
    return json.loads(match.group(1)) if match else None


def write(relative, data):
    path = os.path.join(OUT, relative)
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "w", encoding="utf-8") as file:
        if isinstance(data, (bytes, str)):
            file.write(data if isinstance(data, str) else data.decode())
        else:
            file.write(json.dumps(data, indent=2, ensure_ascii=False) + "\n")


def origin(text):
    """The reference's own origin is production's APP_URL."""
    return text.replace(BASE, APP_URL)


# The query strings of the demo page's parsing table: PHP's intval for
# count and hours (after TrimStrings and ConvertEmptyStringsToNull), and
# FILTER_VALIDATE_BOOLEAN for summary and meta.
QUERY_TABLE = [
    "count=10abc&hours=1.9", "count=1e2&hours=-5", "count=abc&hours=", "count=&hours=0",
    "count=%2012%20&hours=+7", "count=007&hours=0x1A", "count=99999999999999999999&hours=24",
    "count[]=5&hours[]=", "count=3&count=4", "summary=yes&meta=off", "summary=ON&meta=TRUE",
    "summary=&meta=", "summary=0&meta=1", "summary=%20true%20&meta=no", "summary=2&meta=y",
    "summary[]=1", "hours=1e1", "hours=2.5e1",
]

PAGES = [
    ("home", "/"),
    ("demo", "/demo"),
    ("demo-clamped", "/demo?count=9999&hours=9999&summary=1&meta=0"),
    ("demo-trailing-slash", "/demo/"),
    *[(f"demo-query-{index:02}", f"/demo?{query}") for index, query in enumerate(QUERY_TABLE)],
    ("character", "/2112625428"),
    ("character-count-2", "/2112625428?count=2"),
    ("character-hours-1", "/2112625428?hours=1"),
    ("character-options", "/2112625428?count=3&hours=25&summary=1&meta=0"),
    ("character-leading-zeros", "/02112625428"),
    ("character-trailing-slash", "/2112625428/?count=1"),
    ("character-resolved", "/93813310"),
    ("character-attacker", "/95465499?hours=1"),
    ("character-zkillboard-down", "/90000001"),
    ("character-unknown", "/1"),
    ("character-overflow", "/99999999999999999999"),
]

HTTP = [
    ("up", "/up", {}),
    ("not-found", "/nope", {}),
    ("not-found-json", "/nope", {"Accept": "application/json"}),
    ("preview-known", "/characters/2112625428/preview", {"Accept": "application/json"}),
    ("preview-resolved", "/characters/93813310/preview", {"Accept": "application/json"}),
    ("preview-trailing-slash", "/characters/2112625428/preview/", {"Accept": "application/json"}),
    ("preview-unknown", "/characters/1/preview", {"Accept": "application/json"}),
    ("preview-unknown-html", "/characters/1/preview", {}),
    ("preview-not-numeric", "/characters/abc/preview", {"Accept": "application/json"}),
    ("preview-not-numeric-html", "/characters/abc/preview", {}),
    ("horizon", "/horizon", {}),
    ("horizon-api", "/horizon/api/stats", {}),
    ("sanctum-csrf-cookie", "/sanctum/csrf-cookie", {}),
    ("inertia-stale", "/2112625428?count=5", {"X-Inertia": "true", "X-Inertia-Version": "stale",
                                             "X-Requested-With": "XMLHttpRequest"}),
]

DROPPED = {"horizon", "horizon-api", "sanctum-csrf-cookie"}


def record_pages():
    for name, path in PAGES:
        seed()
        write_fakes()
        status, headers, body = request(path)
        page = page_object(body) if status == 200 else None
        recorded = {"path": path, "status": status, "page": page,
                    "characters": characters(), "jobs": jobs(), "requests": http_log()}
        write(f"pages/{name}.json", recorded)
        print(f"page {name}: {status}")


def record_http():
    for name, path, headers in HTTP:
        seed()
        write_fakes()
        status, response_headers, body = request(path, headers)
        recorded = {"path": path, "request_headers": headers, "status": status,
                    "content_type": response_headers.get("Content-Type")}
        if name not in DROPPED:
            location = response_headers.get("X-Inertia-Location")
            if location:
                recorded["x_inertia_location"] = origin(location)
            if "json" in (response_headers.get("Content-Type") or ""):
                recorded["json"] = json.loads(body)
            elif body:
                write(f"http/{name}.html", body)
            recorded["characters"] = characters()
            recorded["requests"] = http_log()
        write(f"http/{name}.json", recorded)
        print(f"http {name}: {status}")


def record_jobs():
    seed()
    write_fakes()
    status, _, _ = request("/2112625428")
    assert status == 200, status
    queued = jobs()
    subprocess.run([RUN, "artisan", "queue:work", "--stop-when-empty", "--tries=1"], check=True,
                   capture_output=True)
    write("jobs.json", {
        "path": "/2112625428",
        "queued": queued,
        "requests": http_log(),
        "killmails": killmails(),
        "characters": characters(),
        "jobs_left": jobs(),
        "failed_jobs": failed_jobs(),
    })
    print(f"jobs: {len(queued)} queued")


def main():
    if len(sys.argv) > 1 and sys.argv[1] == "seed":
        seed()
        write_fakes()
        print(f"seeded at {NOW.isoformat()}")
        return
    write("meta.json", {"now": NOW.strftime("%Y-%m-%dT%H:%M:%SZ")})
    record_pages()
    record_http()
    record_jobs()


if __name__ == "__main__":
    main()
