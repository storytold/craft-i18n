//! What CLDR says of a locale's plural categories, cardinal ("3 pages"), ordinal ("3rd place") and
//! of ranges ("2–3 pages"), read from ICU4X: the rules the selector checks hold a message to.
//!
//! "Reached" always means by the whole numbers 0 to 1000, as `docs/catalog-format.md` says: the
//! categories a translation must have a word for. The cardinal ones are compared with the runtime
//! crate's own form count in [`crate::spec::LocaleSpec::for_locale`], so the two cannot disagree.

use std::collections::BTreeMap;

use icu_locale_core::Locale;
use icu_plurals::{PluralCategory, PluralRules, PluralRulesWithRanges};

/// A CLDR plural category, in CLDR order: the order `@plural` forms are listed in.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Category {
    Zero,
    One,
    Two,
    Few,
    Many,
    Other,
}

impl Category {
    /// Every category, in CLDR order.
    pub const ALL: [Category; 6] = [
        Category::Zero,
        Category::One,
        Category::Two,
        Category::Few,
        Category::Many,
        Category::Other,
    ];

    /// The name a catalog writes: `zero`, `one`, `two`, `few`, `many`, `other`.
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Category::Zero => "zero",
            Category::One => "one",
            Category::Two => "two",
            Category::Few => "few",
            Category::Many => "many",
            Category::Other => "other",
        }
    }

    /// The category a catalog's name stands for.
    #[must_use]
    pub fn from_name(name: &str) -> Option<Category> {
        Category::ALL.into_iter().find(|c| c.name() == name)
    }

    fn from_icu(category: PluralCategory) -> Category {
        match category {
            PluralCategory::Zero => Category::Zero,
            PluralCategory::One => Category::One,
            PluralCategory::Two => Category::Two,
            PluralCategory::Few => Category::Few,
            PluralCategory::Many => Category::Many,
            PluralCategory::Other => Category::Other,
        }
    }

    fn to_icu(self) -> PluralCategory {
        match self {
            Category::Zero => PluralCategory::Zero,
            Category::One => PluralCategory::One,
            Category::Two => PluralCategory::Two,
            Category::Few => PluralCategory::Few,
            Category::Many => PluralCategory::Many,
            Category::Other => PluralCategory::Other,
        }
    }
}

/// The whole numbers the categories are measured over: 0 to this, inclusive.
pub const REACH: u32 = 1000;

/// What one kind of rules, cardinal or ordinal, says of a locale.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Rule {
    /// Every category the locale's rules define, in CLDR order, `other` last. A key of a variant
    /// may be any of them.
    pub defined: Vec<Category>,
    /// The categories the whole numbers 0 to 1000 reach, in CLDR order. A translation needs a word
    /// for each; `other` is `*`.
    pub reached: Vec<Category>,
    /// The whole numbers from 0 to 1000 of each reached category, ascending.
    members: BTreeMap<Category, Vec<u32>>,
}

impl Rule {
    /// The whole numbers from 0 to 1000 that have `category`.
    #[must_use]
    pub fn members(&self, category: Category) -> &[u32] {
        self.members.get(&category).map_or(&[], Vec::as_slice)
    }

    /// Whether exactly one whole number from 0 to 1000 has `category` (Arabic `zero`, `one` and
    /// `two`; Polish `one`): a translation may say that number in words.
    #[must_use]
    pub fn is_single(&self, category: Category) -> bool {
        self.members(category).len() == 1
    }

    /// The categories of the rules that hold exactly one whole number from 0 to 1000.
    #[must_use]
    pub fn single(&self) -> Vec<Category> {
        self.reached
            .iter()
            .copied()
            .filter(|c| self.is_single(*c))
            .collect()
    }

    fn new(rules: &PluralRules) -> Rule {
        let mut members: BTreeMap<Category, Vec<u32>> = BTreeMap::new();
        for n in 0..=REACH {
            members
                .entry(Category::from_icu(rules.category_for(n)))
                .or_default()
                .push(n);
        }
        let mut defined: Vec<Category> = rules.categories().map(Category::from_icu).collect();
        defined.sort();
        defined.dedup();
        let reached: Vec<Category> = members.keys().copied().collect();
        // A category reached is defined; keep the invariant whatever the data says.
        for c in &reached {
            if !defined.contains(c) {
                defined.push(*c);
            }
        }
        defined.sort();
        Rule {
            defined,
            reached,
            members,
        }
    }
}

/// What the plural ranges of a locale say.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RangeRule {
    /// The categories a range can have, over any two categories the locale defines.
    pub defined: Vec<Category>,
    /// The categories a range has when both its ends are whole numbers from 0 to 1000 and it runs
    /// upward, in CLDR order.
    pub reached: Vec<Category>,
}

/// One locale's plural rules.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Plurals {
    pub cardinal: Rule,
    pub ordinal: Rule,
    pub ranges: RangeRule,
}

impl Plurals {
    /// A locale whose numbers are all `other`: the stand-in for one CLDR has no rules for.
    #[must_use]
    pub fn only_other() -> Plurals {
        let rule = Rule {
            defined: vec![Category::Other],
            reached: vec![Category::Other],
            members: BTreeMap::from([(Category::Other, (0..=REACH).collect())]),
        };
        Plurals {
            cardinal: rule.clone(),
            ordinal: rule,
            ranges: RangeRule {
                defined: vec![Category::Other],
                reached: vec![Category::Other],
            },
        }
    }

    /// A locale known to have two cardinal forms, `one` and `other`, whose rules ICU4X's compiled
    /// data does not carry (Central Kurdish, Dhivehi). Which whole numbers are `one` is unknown,
    /// so the stand-in is cautious: no category is single-valued and no exact number covers
    /// `one`, so a message must give `one` a variant that keeps `{n}`. Ordinals are `other`.
    #[must_use]
    pub fn one_and_other() -> Plurals {
        let cardinal = Rule {
            defined: vec![Category::One, Category::Other],
            reached: vec![Category::One, Category::Other],
            members: BTreeMap::new(),
        };
        Plurals {
            cardinal,
            ordinal: Plurals::only_other().ordinal,
            ranges: RangeRule {
                defined: vec![Category::One, Category::Other],
                reached: vec![Category::One, Category::Other],
            },
        }
    }
}

/// The locale ICU4X is asked for, as the runtime crate asks: the language alone (a script or a
/// region does not change a catalog locale's rules), except Portuguese outside Brazil, whose
/// rules are Portugal's.
fn rules_locale(code: &str) -> Result<Locale, String> {
    let id = Locale::try_from_str(code)
        .map_err(|e| format!("{code:?} is not a BCP 47 locale: {e}"))?
        .id;
    let language = id.language.as_str();
    let brazil = id.region.is_some_and(|r| r.as_str() == "BR");
    let asked = if language == "pt" && !brazil {
        "pt-PT"
    } else {
        language
    };
    Locale::try_from_str(asked).map_err(|e| format!("{code:?}: {e}"))
}

/// The plural rules of `code`.
///
/// # Errors
///
/// Returns an error when `code` is not a BCP 47 tag or ICU4X holds no plural data for it.
pub fn plurals(code: &str) -> Result<Plurals, String> {
    let locale = rules_locale(code)?;
    let cardinal = PluralRules::try_new_cardinal((&locale).into())
        .map_err(|e| format!("{code:?}: no cardinal plural rules: {e}"))?;
    let ordinal = PluralRules::try_new_ordinal((&locale).into())
        .map_err(|e| format!("{code:?}: no ordinal plural rules: {e}"))?;
    let ranges = PluralRulesWithRanges::try_new_cardinal((&locale).into())
        .map_err(|e| format!("{code:?}: no plural range rules: {e}"))?;
    let cardinal_rule = Rule::new(&cardinal);
    let range_rule = RangeRule {
        defined: range_categories(&ranges, &cardinal_rule, false),
        reached: range_categories(&ranges, &cardinal_rule, true),
    };
    Ok(Plurals {
        cardinal: cardinal_rule,
        ordinal: Rule::new(&ordinal),
        ranges: range_rule,
    })
}

/// The categories a range between two numbers of `rule` can have, in CLDR order. With `reached`,
/// the numbers are whole numbers from 0 to 1000 and the range runs upward (its start below its
/// end): a category only the number 0 has is never the end of such a range. Otherwise any two
/// categories the locale defines.
fn range_categories(
    ranges: &PluralRulesWithRanges<PluralRules>,
    rule: &Rule,
    reached: bool,
) -> Vec<Category> {
    let ends = if reached {
        &rule.reached
    } else {
        &rule.defined
    };
    let mut found: Vec<Category> = Vec::new();
    for start in ends {
        for end in ends {
            if reached {
                let lowest = rule.members(*start).first();
                let highest = rule.members(*end).last();
                if !lowest.zip(highest).is_some_and(|(lo, hi)| lo < hi) {
                    continue;
                }
            }
            found.push(Category::from_icu(
                ranges.resolve_range(start.to_icu(), end.to_icu()),
            ));
        }
    }
    found.sort();
    found.dedup();
    found
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names(categories: &[Category]) -> Vec<&'static str> {
        categories.iter().map(|c| c.name()).collect()
    }

    #[test]
    fn category_names_round_trip() {
        for c in Category::ALL {
            assert_eq!(Category::from_name(c.name()), Some(c));
        }
        assert_eq!(Category::from_name("One"), None);
        assert_eq!(Category::from_name("*"), None);
        assert_eq!(Category::from_name(""), None);
    }

    #[test]
    fn cardinal_categories_the_numbers_reach() {
        for (locale, reached) in [
            ("ja", vec!["other"]),
            ("de", vec!["one", "other"]),
            ("fr", vec!["one", "other"]),
            ("pl", vec!["one", "few", "many"]),
            ("ru", vec!["one", "few", "many"]),
            ("ar", vec!["zero", "one", "two", "few", "many", "other"]),
            ("pt-br", vec!["one", "other"]),
        ] {
            let p = plurals(locale).unwrap();
            assert_eq!(names(&p.cardinal.reached), reached, "{locale}");
        }
    }

    #[test]
    fn a_category_may_be_defined_and_not_reached() {
        // French and Spanish `many` is for millions; Polish `other` is for fractions: no whole
        // number from 0 to 1000 has either.
        let fr = plurals("fr").unwrap();
        assert!(fr.cardinal.defined.contains(&Category::Many));
        assert!(!fr.cardinal.reached.contains(&Category::Many));
        let pl = plurals("pl").unwrap();
        assert!(pl.cardinal.defined.contains(&Category::Other));
        assert!(!pl.cardinal.reached.contains(&Category::Other));
    }

    #[test]
    fn ordinal_categories_the_numbers_reach() {
        for (locale, reached) in [
            ("de", vec!["other"]),
            ("pl", vec!["other"]),
            ("ar", vec!["other"]),
            ("ja", vec!["other"]),
            ("es", vec!["other"]),
            ("zh-hans", vec!["other"]),
            ("fr", vec!["one", "other"]),
        ] {
            let p = plurals(locale).unwrap();
            assert_eq!(names(&p.ordinal.reached), reached, "{locale}");
        }
    }

    #[test]
    fn single_valued_categories() {
        let ar = plurals("ar").unwrap();
        assert_eq!(names(&ar.cardinal.single()), ["zero", "one", "two"]);
        let pl = plurals("pl").unwrap();
        assert_eq!(names(&pl.cardinal.single()), ["one"]);
        let de = plurals("de").unwrap();
        assert_eq!(names(&de.cardinal.single()), ["one"]);
        let fr = plurals("fr").unwrap();
        assert!(fr.cardinal.single().is_empty(), "0 and 1 are both `one`");
        assert_eq!(fr.cardinal.members(Category::One), [0, 1]);
        assert_eq!(fr.ordinal.members(Category::One), [1]);
        assert_eq!(names(&fr.ordinal.single()), ["one"]);
        let ru = plurals("ru").unwrap();
        assert!(ru.cardinal.single().is_empty());
    }

    #[test]
    fn members_cover_every_number_once() {
        for locale in ["ja", "ar", "pl", "fr", "ru", "cy", "sl"] {
            let p = plurals(locale).unwrap();
            for rule in [&p.cardinal, &p.ordinal] {
                let total: usize = rule.reached.iter().map(|c| rule.members(*c).len()).sum();
                assert_eq!(total, REACH as usize + 1, "{locale}");
            }
        }
    }

    #[test]
    fn ranges_have_their_own_categories() {
        // Where CLDR's table is silent a range takes its end's category: `0–1` is `one` in German
        // and French, and Polish keeps `few` and `many` for ranges.
        for (locale, reached) in [
            ("de", vec!["one", "other"]),
            ("fr", vec!["one", "other"]),
            ("pl", vec!["one", "few", "many"]),
            ("ja", vec!["other"]),
        ] {
            let p = plurals(locale).unwrap();
            assert_eq!(names(&p.ranges.reached), reached, "{locale}");
        }
        let fr = plurals("fr").unwrap();
        assert!(fr.ranges.defined.contains(&Category::Many));
    }

    #[test]
    fn the_ranges_reached_are_ranges_the_locale_defines() {
        for locale in [
            "ja", "de", "fr", "pl", "ru", "ar", "cy", "sl", "lv", "he", "it",
        ] {
            let p = plurals(locale).unwrap();
            assert!(!p.ranges.reached.is_empty(), "{locale}");
            for c in &p.ranges.reached {
                assert!(p.ranges.defined.contains(c), "{locale}: {c:?}");
            }
        }
    }

    #[test]
    fn portuguese_follows_the_runtime_crate() {
        // Brazil's rules put 0 and 1 in `one`; Portugal's only 1.
        assert_eq!(
            plurals("pt-br").unwrap().cardinal.members(Category::One),
            [0, 1]
        );
        assert_eq!(
            plurals("pt-pt").unwrap().cardinal.members(Category::One),
            [1]
        );
        assert_eq!(plurals("pt").unwrap().cardinal.members(Category::One), [1]);
    }

    #[test]
    fn not_a_locale_is_an_error() {
        assert!(plurals("not a locale").is_err());
        assert!(plurals("").is_err());
    }

    #[test]
    fn the_cautious_two_form_locale_has_no_known_members() {
        let p = Plurals::one_and_other();
        assert_eq!(names(&p.cardinal.reached), ["one", "other"]);
        assert!(p.cardinal.single().is_empty());
        assert!(p.cardinal.members(Category::One).is_empty());
        assert_eq!(names(&p.ordinal.reached), ["other"]);
        assert_eq!(names(&p.ranges.reached), ["one", "other"]);
    }

    #[test]
    fn icu_compiled_data_lacks_two_locales_the_runtime_has() {
        // If this fails, ICU4X carries them now: drop `Plurals::one_and_other` and its use.
        for locale in ["ckb", "dv"] {
            let p = plurals(locale).unwrap();
            assert_eq!(names(&p.cardinal.defined), ["other"], "{locale}");
        }
    }

    #[test]
    fn only_other_has_one_category() {
        let p = Plurals::only_other();
        assert_eq!(names(&p.cardinal.reached), ["other"]);
        assert_eq!(
            p.cardinal.members(Category::Other).len(),
            REACH as usize + 1
        );
    }
}
