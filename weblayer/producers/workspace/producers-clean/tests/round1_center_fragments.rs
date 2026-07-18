//! R1 fixture tests — theory-view CENTER section fragments.
//!
//! Pattern: build an input value with Rust constructors, assert its render
//! equals bytes observed at the oracle boundary. Every expected string below
//! is captured program OUTPUT, with its provenance in workspace/QUERIES.log
//! ([S08] issue515 raw envelope, [L03] live macros probe, [L04] live alert
//! probe, [L06] live escape probe, corpus sweeps [S07]–[S12]). Whole-corpus
//! reassembly parity lives in tests/corpus_sweep.rs; integration truth is the
//! corpus web-parity gate.

use producers_clean::html::{
    alert_envelope, escape_text, html_envelope, postprocess_lines, redirect_envelope,
};
use producers_clean::model::{Content, ContentPane, EmptyRender, HeadedBlock, HelpPane};

fn empty() -> Content {
    Content { lines: vec![] }
}

fn lines(ls: &[&str]) -> Content {
    Content {
        lines: ls.iter().map(|s| s.to_string()).collect(),
    }
}

// Compiles + constructs: proves the input model is usable.
#[test]
fn model_constructs() {
    let pane = ContentPane {
        title: "Tactics".into(),
        blocks: vec![HeadedBlock {
            heading: "Tactic(s)".into(),
            body: empty(),
            when_empty: EmptyRender::Keep,
        }],
    };
    assert_eq!(pane.title, "Tactics");
    assert!(pane.blocks[0].body.is_empty());
}

// The escape set forced through the help env line via a metachar filename
// [L06]: & " < > ' escape; backslash passes through.
#[test]
fn escape_text_observed_set() {
    assert_eq!(escape_text(r#"esc&"<>'probe"#), "esc&amp;&quot;&lt;&gt;&#39;probe");
    assert_eq!(escape_text(r"back\slash"), r"back\slash");
    assert_eq!(escape_text("plain, text."), "plain, text.");
}

// Per-line postprocess [S03][S10]: every line (empty and last included) gets
// `<br/>\n`; leading spaces become `&nbsp;` runs; interior spaces survive.
#[test]
fn postprocess_lines_rules() {
    assert_eq!(postprocess_lines(""), "<br/>\n");
    assert_eq!(postprocess_lines("a"), "a<br/>\n");
    assert_eq!(postprocess_lines("a\n\nb"), "a<br/>\n<br/>\nb<br/>\n");
    assert_eq!(
        postprocess_lines("   [ x ] --> [ y ]"),
        "&nbsp;&nbsp;&nbsp;[ x ] --> [ y ]<br/>\n"
    );
    // Interior runs stay spaces; only the leading run converts.
    assert_eq!(postprocess_lines("  a  b"), "&nbsp;&nbsp;a  b<br/>\n");
}

// The tactic pane is the simplest complete fragment: one always-present block
// with an empty body. Expected bytes = the raw captured envelope, verbatim
// (issue515 main/tactic [S08]; identical for the lemma-less live theory [L06]).
#[test]
fn tactic_pane_empty_body() {
    let pane = ContentPane {
        title: "Tactics".into(),
        blocks: vec![HeadedBlock {
            heading: "Tactic(s)".into(),
            body: empty(),
            when_empty: EmptyRender::Keep,
        }],
    };
    let out = producers_clean::render_content_pane(&pane);
    assert_eq!(
        out,
        "{\"html\":\"<h2>Tactic(s)</h2><br/>\\n<p class=\\\"monospace rules\\\"></p><br/>\\n\",\"title\":\"Tactics\"}"
    );
}

// The message pane always emits three sections in the observed heading order
// [S07]; each h2 and each p sits on its own postprocessed line, the p-opener
// glued to the first body line [S03].
#[test]
fn message_pane_three_sections_framing() {
    let pane = ContentPane {
        title: "Message theory".into(),
        blocks: vec![
            HeadedBlock {
                heading: "Signature".into(),
                body: lines(&["SIG1", "SIG2"]),
                when_empty: EmptyRender::Keep,
            },
            HeadedBlock {
                heading: "Construction Rules".into(),
                body: lines(&["CTOR"]),
                when_empty: EmptyRender::Keep,
            },
            HeadedBlock {
                heading: "Deconstruction Rules".into(),
                body: lines(&["DTOR"]),
                when_empty: EmptyRender::Keep,
            },
        ],
    };
    let out = producers_clean::render_content_pane(&pane);
    let expected_html = "<h2>Signature</h2><br/>\n\
                         <p class=\"monospace rules\">SIG1<br/>\n\
                         SIG2</p><br/>\n\
                         <h2>Construction Rules</h2><br/>\n\
                         <p class=\"monospace rules\">CTOR</p><br/>\n\
                         <h2>Deconstruction Rules</h2><br/>\n\
                         <p class=\"monospace rules\">DTOR</p><br/>\n";
    assert_eq!(out, html_envelope("Message theory", expected_html));
    assert!(out.starts_with("{\"html\":\"<h2>Signature</h2><br/>\\n"));
}

// The rules pane without macros/restrictions: the empty macros slot leaves one
// leading blank line (all 81 corpus captures start `<br/>\n` [S07]); the empty
// restrictions section vanishes without residue (pane ends right after the MSR
// block [S10]). Title is the CONSTANT observed string [S07].
#[test]
fn rules_pane_no_macros_no_restrictions() {
    let pane = ContentPane {
        title: "Multiset rewriting rules and restrictions".into(),
        blocks: vec![
            HeadedBlock {
                heading: "Macros".into(),
                body: empty(),
                when_empty: EmptyRender::BlankLine,
            },
            HeadedBlock {
                heading: "Fact Symbols with Injective Instances".into(),
                body: lines(&["None"]),
                when_empty: EmptyRender::Keep,
            },
            HeadedBlock {
                heading: "Multiset Rewriting Rules".into(),
                body: lines(&["RULE A", "   BODY"]),
                when_empty: EmptyRender::Keep,
            },
            HeadedBlock {
                heading: "Restrictions of the Set of Traces".into(),
                body: empty(),
                when_empty: EmptyRender::Omit,
            },
        ],
    };
    let out = producers_clean::render_content_pane(&pane);
    let expected_html = "<br/>\n\
                         <h2>Fact Symbols with Injective Instances</h2><br/>\n\
                         <p class=\"monospace rules\">None</p><br/>\n\
                         <h2>Multiset Rewriting Rules</h2><br/>\n\
                         <p class=\"monospace rules\">RULE A<br/>\n\
                         &nbsp;&nbsp;&nbsp;BODY</p><br/>\n";
    assert_eq!(
        out,
        html_envelope("Multiset rewriting rules and restrictions", expected_html)
    );
}

// With macros the pane starts DIRECTLY with the Macros block — no leading
// blank. Expected prefix bytes observed live (MacroGlobalVarNSPK3 [L03]).
#[test]
fn rules_pane_with_macros_starts_with_macros_block() {
    let pane = ContentPane {
        title: "Multiset rewriting rules and restrictions".into(),
        blocks: vec![
            HeadedBlock {
                heading: "Macros".into(),
                body: lines(&[
                    "<span class=\"hl_keyword\">macros:</span> globalVar( ) =  $R",
                ]),
                when_empty: EmptyRender::BlankLine,
            },
            HeadedBlock {
                heading: "Fact Symbols with Injective Instances".into(),
                body: lines(&["None"]),
                when_empty: EmptyRender::Keep,
            },
        ],
    };
    let out = producers_clean::render_content_pane(&pane);
    let expected_html = "<h2>Macros</h2><br/>\n\
                         <p class=\"monospace rules\"><span class=\"hl_keyword\">macros:</span> globalVar( ) =  $R</p><br/>\n\
                         <h2>Fact Symbols with Injective Instances</h2><br/>\n\
                         <p class=\"monospace rules\">None</p><br/>\n";
    assert_eq!(
        out,
        html_envelope("Multiset rewriting rules and restrictions", expected_html)
    );
}

// Restrictions present: directly follows the MSR block, no blank line at the
// junction [S11].
#[test]
fn rules_pane_restrictions_junction_no_blank() {
    let pane = ContentPane {
        title: "Multiset rewriting rules and restrictions".into(),
        blocks: vec![
            HeadedBlock {
                heading: "Multiset Rewriting Rules".into(),
                body: lines(&["R"]),
                when_empty: EmptyRender::Keep,
            },
            HeadedBlock {
                heading: "Restrictions of the Set of Traces".into(),
                body: lines(&["REST"]),
                when_empty: EmptyRender::Omit,
            },
        ],
    };
    let out = producers_clean::render_content_pane(&pane);
    assert!(out.contains(
        "R</p><br/>\\n<h2>Restrictions of the Set of Traces</h2><br/>\\n"
    ));
}

// Help pane, warning-free: env line ends `) </p>` [S09][L05][L06]; the origin
// is entity-escaped by the producer (metachar filename probe [L06]); the fixed
// static block follows; no trailing newline [S10]; title `Theory: NAME` [S07].
#[test]
fn help_pane_no_warnings() {
    let help = HelpPane {
        theory_name: "EscProbe".into(),
        load_time: "15:23:01".into(),
        origin: "Local \"/tmp/x/thy/esc&\\\"<>'probe.spthy\"".into(),
        wf_banner_html: String::new(),
    };
    let out = producers_clean::render_help_pane(&help);
    let expected_env_line = "<p>Theory: EscProbe (Loaded at 15:23:01 from \
                             Local &quot;/tmp/x/thy/esc&amp;\\&quot;&lt;&gt;&#39;probe.spthy&quot;) </p>";
    let expected = html_envelope(
        "Theory: EscProbe",
        &format!(
            "{}{}",
            expected_env_line,
            producers_clean::section::HELP_STATIC_HTML
        ),
    );
    assert_eq!(out, expected);
    assert!(!out.ends_with("\\n\"}"));
    // The static block is the observed invariant [S09]: spot-check its head
    // and tail bytes.
    assert!(producers_clean::section::HELP_STATIC_HTML
        .starts_with("<div id=\"help\"><h3>Quick introduction</h3>"));
    assert!(producers_clean::section::HELP_STATIC_HTML.ends_with("</table></div></p>"));
}

// Help pane with a banner: raw passthrough between `) ` and `</p>` [S09].
#[test]
fn help_pane_with_banner() {
    let help = HelpPane {
        theory_name: "issue515".into(),
        load_time: "23:53:03".into(),
        origin: "Local \"/tmp/t/thy/issue515.spthy\"".into(),
        wf_banner_html: "<div class=\"wf-warning\">\nWARNING!<br/>\n</div>".into(),
    };
    let out = producers_clean::render_help_pane(&help);
    assert!(out.contains(
        "issue515.spthy&quot;) <div class=\\\"wf-warning\\\">\\nWARNING!<br/>\\n</div></p><div id=\\\"help\\\">"
    ));
    assert!(out.contains("\"title\":\"Theory: issue515\"}"));
}

// The three envelope shapes, byte-pinned: {html,title} [S08], {redirect}
// (corpus [S07]), {alert} (live [L04]). JSON escaping = \" \\ \n \t [S09].
#[test]
fn envelope_shapes() {
    assert_eq!(
        html_envelope("T", "a\"b\\c\nd\te"),
        "{\"html\":\"a\\\"b\\\\c\\nd\\te\",\"title\":\"T\"}"
    );
    assert_eq!(
        redirect_envelope("/thy/trace/2/overview/proof/simp"),
        "{\"redirect\":\"/thy/trace/2/overview/proof/simp\"}"
    );
    assert_eq!(
        alert_envelope("Can't delete the given theory path!"),
        "{\"alert\":\"Can't delete the given theory path!\"}"
    );
}
