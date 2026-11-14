//! The reviewed Ask Agent labels must survive catalog updates without losing the two-line
//! ribbon label, switching the technical agent term, or silently falling back to English for
//! controls added to WordCraft's conversation and review flow.

use std::collections::BTreeMap;

use craft_i18n_tests::{catalog::read_catalog, locale::plural_forms, repo_root};

fn word_rows(locale: &str) -> BTreeMap<String, String> {
    let mut rows = BTreeMap::new();
    for name in ["common.tsv", "wordcraft.tsv"] {
        let path = repo_root().join("locales").join(locale).join(name);
        let (catalog, problems) = read_catalog(
            &std::fs::read_to_string(&path).unwrap(),
            plural_forms(locale).unwrap(),
        );
        assert!(problems.is_empty(), "{}: {problems:?}", path.display());
        for row in catalog.into_iter().filter(|row| row.context.is_empty()) {
            rows.insert(row.source, row.translation);
        }
    }
    rows
}

#[test]
fn ribbon_keeps_the_reviewed_technical_agent_terms() {
    // `Ajana` is the Turkish dative of `Ajan`, agreeing with existing `Ajanlar`.
    for (locale, term) in [
        ("vi", "tác tử"),
        ("zh-hans", "智能体"),
        ("hu", "Ágens"),
        ("tr", "Ajan"),
    ] {
        let rows = word_rows(locale);
        let label = &rows["Ask\nAgent"];
        assert_eq!(label.matches('\n').count(), 1, "{locale}: {label:?}");
        assert!(
            label.contains(term),
            "{locale}: use the reviewed technical agent term {term:?}, got {label:?}"
        );
    }
    assert_eq!(
        word_rows("vi")["Ask\nAgent"].replace('\n', " "),
        word_rows("vi")["Ask Agent"]
    );
}

#[test]
fn completed_word_locales_cover_the_conversation_and_review_controls() {
    for locale in ["fr", "hr", "pl", "vi"] {
        let rows = word_rows(locale);
        for key in [
            "Actions",
            "Agent",
            "Any",
            "Ask Agent",
            "Ask\nAgent",
            "Choose a starting instruction, then edit and send",
            "Close",
            "Conversation",
            "Current instruction",
            "Format",
            "Held by Agent",
            "Keep",
            "Keep tool edits and notes. Proposals are not applied.",
            "Proposal (not applied)",
            "No agent is listening: ask your agent to wait for requests",
            "Reply to refine…",
            "Send",
            "Start WordCraft with --control to let an agent connect",
            "The configured agent launcher must be an absolute executable path",
            "The document changed since the agent's edits; use Undo in the Edit menu",
            "Undo",
            "Undo all of the agent's edits",
            "Waiting for an agent…",
            "What should the agent do?",
            "Working on it…",
            "You",
            "Shorten",
            "Fix grammar",
            "Professional",
            "Rewrite",
            "Friendly",
            "Concise",
            "English",
            "Translate",
            "French",
            "Summarize",
        ] {
            assert!(
                rows.contains_key(key),
                "{locale}: missing {key:?}; WordCraft would show English"
            );
        }
    }
}
