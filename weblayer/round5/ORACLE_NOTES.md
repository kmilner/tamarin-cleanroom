# Round-5 staged captures — del/path and verify route families

The round-4 conclusion that `del/path` and `verify` are absent in the reference
was a probe-shape artifact: both routes take a further TWO-segment sub-path.
The four JSON files here are genuine captured reference responses (dirty-room
staged; capture metadata below records the requests that produced them):

| Capture | Request that produced it |
|---------|--------------------------|
| del_path.json | GET /thy/trace/&lt;idx&gt;/del/path/lemma/&lt;lemma-name&gt; |
| del_path_bad.json | GET /thy/trace/&lt;idx&gt;/del/path/rules (an undeletable path) |
| verify.json | GET /thy/trace/&lt;idx&gt;/verify/lemma/&lt;lemma-name&gt; |
| verify_proof.json | GET /thy/trace/&lt;idx&gt;/verify/proof/&lt;proof-path&gt; |

Known from the captures alone: del/path on a lemma returns a `{redirect}`
envelope pointing at an overview view and allocates a fresh version index;
del/path on an undeletable path returns an `{alert}`; verify/lemma returns an
`{html,title}` envelope; verify/proof returns a `{redirect}`. Everything else
(status codes, headers, exact redirect targets across states, index
allocation semantics, diff-mode variants, method restrictions, error shapes
for missing/bogus sub-paths) must be established by live probing with these
shapes.
