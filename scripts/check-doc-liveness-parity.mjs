#!/usr/bin/env node
// check-doc-liveness-parity.mjs — the drift axis that placement choice ⒜ (copy) requires.
//
// AGENTS.md §Documented-command liveness decided (2026-09-10) that the weekly job
// .github/workflows/doc-liveness.yml runs the documented commands as a verbatim COPY of
// this repo's doc. A copy drifts, so this checker diffs the two sides — both directions —
// on every PR (an engine-contract.yml step, like the other checkers in scripts/):
//
//   doc side:      every executable line inside AGENTS.md's fenced sh blocks
//                  (fence = a line of optional whitespace + ```sh … closed by ```),
//                  with trailing aligned comments stripped (2+ spaces then #).
//   workflow side: every executable line between `# DOC-COPY BEGIN` / `# DOC-COPY END`
//                  in the workflow, plus every `# NOT-RUN: <line> — <why>` declaration
//                  (the declared line counts as covered; the reason stays in the
//                  workflow, next to the copy — the runner-block NOT-RUN precedent,
//                  so this checker holds no classification of its own).
//
// The two multisets must be EQUAL. A doc edit without the workflow (or vice versa)
// fails here, which is the point: the alias layer died silently precisely because
// nothing tied the documented notation to what actually runs.
//
// Exit: 0 = in sync · 1 = drift (both diffs printed) · 2 = structural error (missing
// file / unbalanced markers) — 2 is "could not measure", never "in sync".

import { readFileSync } from "node:fs";

const DOC = "AGENTS.md";
const WF = ".github/workflows/doc-liveness.yml";

function die(msg) {
  console.error(`check-doc-liveness-parity: ${msg}`);
  process.exit(2);
}

// Shared normalization — MUST be identical for both sides, or the compare lies.
// Trailing-comment rule: strip from the first run of 2+ spaces followed by '#'.
// (Doc comments are column-aligned; a single space before '#' never introduces one —
// measured over every fenced line at adoption time. '#' inside quoted strings, e.g.
// grep -H '^## \[', is not preceded by 2 spaces and survives.)
const normalize = (line) => line.replace(/\s{2,}#.*$/, "").trim();
const isExecutable = (line) => line !== "" && !line.startsWith("#");

function docLines(text) {
  const out = [];
  let inFence = false;
  let fenceIsSh = false;
  for (const raw of text.split("\n")) {
    const fence = raw.match(/^\s*```(\S*)\s*$/);
    if (fence) {
      if (!inFence) {
        inFence = true;
        fenceIsSh = fence[1] === "sh" || fence[1] === "bash";
      } else {
        inFence = false;
        fenceIsSh = false;
      }
      continue;
    }
    if (inFence && fenceIsSh) {
      const n = normalize(raw);
      if (isExecutable(n)) out.push(n);
    }
  }
  if (inFence) die(`${DOC}: unbalanced code fence`);
  return out;
}

function workflowLines(text) {
  const covered = [];
  let inRegion = false;
  let regions = 0;
  for (const raw of text.split("\n")) {
    const t = raw.trim();
    if (t === "# DOC-COPY BEGIN") {
      if (inRegion) die(`${WF}: nested DOC-COPY BEGIN`);
      inRegion = true;
      regions++;
      continue;
    }
    if (t === "# DOC-COPY END") {
      if (!inRegion) die(`${WF}: DOC-COPY END without BEGIN`);
      inRegion = false;
      continue;
    }
    const notRun = t.match(/^# NOT-RUN: (.+?) — /);
    if (notRun) {
      covered.push(normalize(notRun[1]));
      continue;
    }
    if (inRegion) {
      const n = normalize(raw);
      if (isExecutable(n)) covered.push(n);
    }
  }
  if (inRegion) die(`${WF}: unclosed DOC-COPY region`);
  if (regions === 0) die(`${WF}: no DOC-COPY region found — the copy vanished`);
  return covered;
}

let docText, wfText;
try {
  docText = readFileSync(DOC, "utf8");
} catch {
  die(`${DOC}: unreadable`);
}
try {
  wfText = readFileSync(WF, "utf8");
} catch {
  die(`${WF}: unreadable — the weekly job AGENTS.md §Documented-command liveness names is gone`);
}

// Multiset diff (duplicates matter: `done` legitimately appears twice on each side).
const count = (arr) => {
  const m = new Map();
  for (const x of arr) m.set(x, (m.get(x) ?? 0) + 1);
  return m;
};
const doc = count(docLines(docText));
const wf = count(workflowLines(wfText));

const onlyDoc = [];
const onlyWf = [];
for (const [line, n] of doc) {
  const d = n - (wf.get(line) ?? 0);
  for (let i = 0; i < d; i++) onlyDoc.push(line);
}
for (const [line, n] of wf) {
  const d = n - (doc.get(line) ?? 0);
  for (let i = 0; i < d; i++) onlyWf.push(line);
}

if (onlyDoc.length === 0 && onlyWf.length === 0) {
  const total = [...doc.values()].reduce((a, b) => a + b, 0);
  console.log(`doc-liveness parity: OK — ${total} documented line(s), workflow copy + NOT-RUN cover all of them, nothing extra`);
  process.exit(0);
}

if (onlyDoc.length) {
  console.error(`DOC ONLY — ${DOC} names these, but ${WF} neither runs nor NOT-RUN-declares them:`);
  for (const l of onlyDoc) console.error(`  + ${l}`);
  console.error(`  ⇒ add the line to a DOC-COPY region, or a '# NOT-RUN: <line> — <why>' beside the copy, in the same PR.`);
}
if (onlyWf.length) {
  console.error(`WORKFLOW ONLY — ${WF} carries these, but ${DOC} no longer documents them:`);
  for (const l of onlyWf) console.error(`  - ${l}`);
  console.error(`  ⇒ the doc moved (or the copy drifted): update the workflow copy to match the doc.`);
}
process.exit(1);
