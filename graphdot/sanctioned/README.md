# Sanctioned third-party material — graphdot cluster

## pretty-1.1.3.6/ (staged 2026-07-18)

The Haskell `pretty` package, version 1.1.3.6 — the pretty-printing library
(Text.PrettyPrint.HughesPJ) that GHC 9.6.x ships as a boot library and that
the reference binary's toolchain (stack lts-22.44 / GHC 9.6.7) uses.

- Source: https://hackage.haskell.org/package/pretty-1.1.3.6
  (tarball sha256 eb12cc23cbaf7d3f3e2a5e804e2e0ddb2a2d98d3ad456617d1b87fd3951d66e8;
  src/ + LICENSE + pretty.cabal staged verbatim)
- License: BSD-style (Glasgow Haskell Compiler License) — NOT GPL, NOT part of
  tamarin-prover. It is already classed as EXTERNAL by the campaign's header
  tooling.

CLEAN-ROOM STATUS: this directory is SANCTIONED reading material for clean-room
implementers in this cluster. Implementing a faithful equivalent of this
library's layout algorithms (Doc, sep/fsep/fillSep, fitting/ribbon logic) from
this BSD source is permitted and does not taint the clean room — the forbidden
set remains tamarin-prover's own sources (*.hs outside this directory) and the
tamarin-rs Rust port. Behavioral parameters that tamarin itself feeds into
these combinators (widths, ribbons, indents) are NOT in this library and must
still be derived from black-box probes as before.
