# Short initial sanity integration suite

The retained suite is debug/sanity.json; tooling/integration.json exposes the same
two checks with an adapter path relative to tooling/. Both use sanity_adapter.py
and scenarios/sanity.py. No full-game scenarios, routes or setup caches remain.

From /root/ducktales, run with a new evidence directory:

```sh
python3 /root/romatlas/scripts/qa_harness.py --manifest debug/tooling/integration.json --suite initial --expectation original-fidelity --include-original-fidelity-sanity --evidence debug/artifacts/NEW-RUN
```

Existing sanity-results.json and sanity-review.md link retained historical title
and Amazon-motion evidence, including a superseded failed sanity attempt. These
checks use semantic landmark alignment and selected presentation/audio review;
they do not establish exact-frame equality or current-build acceptance. Read
identities.json and reviews before deciding whether evidence remains applicable.

The updated debug/goal.md and acceptance.json require only the short smoke and
initial exact-frame comparison suite. plan.json and qa.json are fresh pending
records: no new test execution or acceptance is claimed by this cleanup.
