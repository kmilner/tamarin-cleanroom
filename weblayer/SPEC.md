# Clean-room task: web UI layer (HTML/JSON responses)

Build, from BLACK-BOX BEHAVIOR ONLY, a standalone crate `web-clean` in
workspace/ that reproduces the HTTP response bodies of the tamarin-prover
interactive web UI: page HTML, JSON envelopes, and the URL route structure,
byte-identically where the captures show deterministic output.

## Oracles
- oracle/captured_responses/ -> 81 crawl manifests (url -> response) of the
  Haskell web UI across whole theories. Primary corpus; explore the format.
- oracle/hs_server.sh start <file.spthy> <port> for live probing (curl any
  route; discover routes from captures and links inside returned HTML).
  Example inputs: ../wellformedness/oracle/examples/.

## Deliverables
1. workspace/BEHAVIOR.md — observed spec: route map, response envelope
   rules, HTML template structure per page type (as observed output text),
   escaping rules, state-dependence.
2. `web-clean`: template/render functions producing the response bodies from
   YOUR OWN input model (design from observations; integration adapters come
   later). Tests compare rendered output against captured responses.
3. workspace/QUERIES.log as per protocol.
Dependencies: std + serde/serde_json only.
