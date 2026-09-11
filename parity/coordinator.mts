/**
 * Reference coordinator harness.
 *
 * Reads raw shard wires dumped by `parity_runner --dump-wires` and runs the
 * reference app's own merge/finalize functions on them, producing the exact
 * `AggregatedResult` JSON the React/Wasm app would show for default filters.
 *
 * Usage (from the reference repo, so tsx resolves):
 *   cd ../pm2-log-analyzer && npx tsx ../pm2-log-analyzer-rust/parity/coordinator.mts <wiresDir> [mode]
 */
import fs from "node:fs";
import path from "node:path";

import {
  aggregateCron,
  finalizeDailyStats,
  finalizeHourlyStats,
  finishApiFromPartials,
  mergeDailyPartials,
  mergeHourlyPartials,
} from "../../pm2-log-analyzer/src/parser/index.ts";
import {
  decodeCronWire,
  decodeDailyWire,
  decodeDatesWire,
  decodeHourlyWire,
  decodePm2Partial,
  decodeUnmatchedWire,
  methodsFromMask,
} from "../../pm2-log-analyzer/src/wasm/decodePartial.ts";
import type {
  CronEventCompact,
  DaySummary,
  HourlyBucket,
  HourlyPartial,
  NormalizeMode,
  ParseOptions,
} from "../../pm2-log-analyzer/src/parser/index.ts";
import type { AggPartial, DailyPartial as DailyPartialT } from "../../pm2-log-analyzer/src/parser/index.ts";

const wiresDir = process.argv[2];
if (!wiresDir) {
  console.error("usage: coordinator.mts <wiresDir> [mode]");
  process.exit(2);
}
const mode = (process.argv[3] as NormalizeMode | undefined) ?? "collapseIds";

const opts: ParseOptions = {
  normalizeMode: mode,
  methodFilter: null,
  statusFamily: "all",
  minMs: 0,
  cronQuery: "",
  cronMinMs: 0,
  cronShowFailedOnly: false,
  dateFilter: null,
};

const read = (file: string) => new Uint8Array(fs.readFileSync(path.join(wiresDir, file)));

const meta = JSON.parse(fs.readFileSync(path.join(wiresDir, "meta.json"), "utf8")) as {
  shards: { index: number; hitCount: number; unmatchedCount: number; methodsMask: number }[];
};
const shards = [...meta.shards].sort((a, b) => a.index - b.index);

let hitCount = 0;
let unmatchedCount = 0;
let methodsMask = 0;
const cronEvents: CronEventCompact[] = [];
const allDates: string[] = [];
const hourlyPartials: HourlyPartial[] = [];
const dailyPartials: DailyPartialT[] = [];
const unmatchedSample: string[] = [];

for (const s of shards) {
  hitCount += s.hitCount;
  unmatchedCount += s.unmatchedCount;
  methodsMask |= s.methodsMask;
  hourlyPartials.push(decodeHourlyWire(read(`shard${s.index}.hourly`)));
  cronEvents.push(...decodeCronWire(read(`shard${s.index}.cron`)));
  allDates.push(...decodeDatesWire(read(`shard${s.index}.dates`)));
  dailyPartials.push(decodeDailyWire(read(`shard${s.index}.daily`)));
  for (const line of decodeUnmatchedWire(read(`shard${s.index}.unmatched`))) {
    if (unmatchedSample.length >= 40) break;
    unmatchedSample.push(line);
  }
}

const hourlyStats: HourlyBucket[] = finalizeHourlyStats(mergeHourlyPartials(hourlyPartials));
const methods = methodsFromMask(methodsMask);
const dates = Array.from(new Set(allDates)).sort();
const dailyStats: DaySummary[] = finalizeDailyStats(mergeDailyPartials(dailyPartials));

const partials: AggPartial[] = [];
let totalMatched = 0;
let totalUnmatched = 0;
for (const s of shards) {
  const decoded = decodePm2Partial(read(`shard${s.index}.partial`));
  totalMatched += decoded.matched;
  totalUnmatched += decoded.unmatched;
  partials.push(decoded.partial);
}

const { api, summary } = finishApiFromPartials(partials, opts, {
  count: totalMatched,
  unmatchedCount: totalUnmatched,
});
const cron = aggregateCron(cronEvents, opts);

let starts = 0;
let dones = 0;
let fails = 0;
for (const e of cronEvents) {
  if (e.event === "start") starts++;
  else if (e.event === "done") dones++;
  else fails++;
}

const result = {
  api,
  cron,
  summary,
  cronSummary: {
    starts,
    dones,
    fails,
    jobs: cron.length,
    slowestRun: cron.reduce((m, r) => Math.max(m, r.maxMs), 0),
  },
  hourlyStats,
  methods,
  unmatchedSample,
  unmatchedCount,
  dates,
  dailyStats,
};

console.log(JSON.stringify(result));
