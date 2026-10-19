# Engine friction log

One row per friction review or 2×–3× note (ADR-0092, the
`engine-friction` skill): what the job was, how big we expected it to be,
how big it was, and what we did about it. Newest last. Three or more
"accept" rows on the same engine area trigger a review; sweeps (1153)
read this table.

| Date | Ticket | The job | Estimate | Measured | Ratio | Outcome | Follow-up |
| ---- | ------ | ------- | -------- | -------- | ----- | ------- | --------- |
| 2026-10-18 | 1151 | count production Rust lines per crate in a diff (`cargo xtask code-lines`) | 180 | 312 | 1.7× | under 2× (the log's first row, asked for by the ticket; not a review) | — |
