use garde::Validate;
use once_cell::sync::Lazy;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use strum_macros::EnumIter;

/// Human history time period of a world wonder
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, EnumIter, JsonSchema)]
pub enum TimePeriod {
    Prehistoric,
    Ancient,
    Classical,
    PostClassical,
    EarlyModern,
    Modern,
}
// Derive time period from a signed integer representation of a wonder's build year (negative = BCE)
// Source for time period break points: https://en.wikipedia.org/wiki/Human_history
impl From<i16> for TimePeriod {
    fn from(value: i16) -> Self {
        match value {
            ..=-3000 => TimePeriod::Prehistoric,
            -2999..=-800 => TimePeriod::Ancient,
            -799..=500 => TimePeriod::Classical,
            501..=1500 => TimePeriod::PostClassical,
            1501..=1800 => TimePeriod::EarlyModern,
            1801.. => TimePeriod::Modern,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, JsonSchema, PartialEq, Validate)]
pub struct Links {
    #[garde(url, prefix("https://en.wikipedia.org/wiki/"))]
    pub wiki: String,
    #[garde(url, prefix("https://www.britannica.com"))]
    pub britannica: Option<String>,
    #[garde(url, prefix("https://www.google.com/maps/place"))]
    pub google_maps: Option<String>,
    #[garde(url, prefix("https://www.tripadvisor.com"))]
    pub trip_advisor: Option<String>,
    #[garde(length(min = 2), inner(url, prefix("https")))]
    pub images: Vec<String>,
}

#[derive(
    Clone, Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, EnumIter, JsonSchema,
)]
pub enum Category {
    /// Wonder is one of the "7 Wonders of the Ancient World".
    SevenWonders,
    /// Wonder is one of the "7 Wonders of the Modern World" elected by the American Society of Civil Engineers in 1994.
    SevenModernWonders,
    /// Wonder is one of the "New 7 Wonders of the World" elected by online votes via an initiative by the Swiss corporation New7Wonders Foundation
    SevenNewWonders,
    /// Wonder can be found in the video game "Civilization V".
    Civ5,
    /// Wonder can be found in the video game "Civilization VI".
    Civ6,
}

#[derive(Serialize, Deserialize, Debug, JsonSchema, PartialEq, Validate)]
#[garde(allow_unvalidated)]
pub struct Wonder {
    #[garde(length(min = 3, max = 150))]
    pub name: String,
    /// Short summary of a world wonder and what it is/was.
    #[garde(length(min = 50, max = 400))]
    pub summary: String,
    /// Location / suspected location of a world wonder or its remains.
    #[garde(length(min = 3, max = 150))]
    pub location: String,
    /// Year / suspected year the wonder was completed.
    pub build_year: i16,
    /// Human history time period that the completion of the world wonder corresponds to.
    /// Derived from the build year
    pub time_period: TimePeriod,
    #[garde(dive)]
    pub links: Links,
    pub categories: Vec<Category>,
}

/// All wonders, read from `data.json`
pub static WONDERS: Lazy<Vec<Wonder>> = Lazy::new(|| {
    serde_json::from_str(include_str!("../data.json"))
        .map_err(|e| panic!("Encountered error while parsing JSON into wonders vec: {e:?}"))
        .unwrap()
});

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use chrono::prelude::*;
    use clearurls::UrlCleaner;

    use super::*;

    const CONTINENTS: [&str; 6] = [
        "Africa",
        "Asia",
        "Europe",
        "North America",
        "Oceania",
        "South America",
    ];

    macro_rules! assert_valid_text {
        ($val: ident) => {
            assert_eq!(
                $val.trim().len(),
                $val.len(),
                "{} contains trailing/leading whitespace:\n{}",
                stringify!($val),
                $val
            );

            let mut consecutive = false;
            for char in ($val).chars() {
                if char.is_whitespace() {
                    assert_eq!(
                        char,
                        ' ',
                        "{} contains non-space whitespace:\n{}",
                        stringify!($val),
                        $val
                    );
                    assert!(
                        !consecutive,
                        "{} contains consecutive spaces:\n{}",
                        stringify!($val),
                        $val
                    );
                    consecutive = true;
                } else {
                    consecutive = false;
                }
            }
        };
    }

    #[test]
    fn validate_wonders_data() {
        assert!(WONDERS.len() > 0);

        // Validate data using `garde`
        WONDERS.iter().for_each(|w| w.validate().unwrap());

        // To check for duplicate names
        let mut seen_names = HashSet::new();
        // To check for duplicate links
        let mut seen_links = HashSet::new();

        // Current year
        let year = Utc::now().year();

        // URL cleaner
        let cleaner = UrlCleaner::from_embedded_rules().expect("Could not create URL cleaner");

        WONDERS.iter().for_each(
            |Wonder {
                name,
                location,
                summary,
                build_year,
                time_period,
                links: Links {
                    wiki,
                    britannica,
                    google_maps,
                    trip_advisor,
                    images
                },
                categories,
                ..
            }| {
                assert_valid_text!(name);
                assert_valid_text!(location);
                assert_valid_text!(summary);

                assert!(
                    location.contains(',') &&
                        CONTINENTS.contains(&location.rsplit_once(',').unwrap().1.trim_start()),
                    "Location must define a continent:\n{location}"
                );
                assert!(summary.ends_with('.') || summary.ends_with('!'), "Summary must end with proper punctuation:\n{summary}");

                // Unique name
                assert!(!seen_names.contains(name.as_str()), "Duplicate name: '{name}'");
                seen_names.insert(name.as_str());

                // Build year + time period
                assert!(*build_year as i32 <= year, "Build year exceeds current calendar year: {build_year}");
                let expected_time_period = TimePeriod::from(*build_year);
                assert_eq!(
                    time_period,
                    &expected_time_period,
                    "Time period '{time_period:?}' does not match year '{build_year}'. Expected: {expected_time_period:?}",
                );

                // CATEGORIES
                let mut categories_clone = categories.clone();
                categories_clone.sort();
                categories_clone.dedup();
                assert_eq!(categories_clone.len(), categories.len(), "There are duplicate categories: {categories:?}");

                // LINKS
                let mut all_links: Vec<&str> = Vec::with_capacity(images.len() + 4);

                // Wiki link
                all_links.push(wiki.as_str());
                assert!(!wiki.contains('#'), "Selecting specific element in wiki page: {wiki}");

                // Other links (`Option` values)
                [britannica, google_maps, trip_advisor].into_iter().for_each(|l| {
                    if let Some(l) = l {
                        assert!(!l.contains('#'), "Selecting specific element in link: {l}");
                        assert!(!l.contains('?'), "Passing query parameters in link: {l}");
                        all_links.push(l);
                    };
                });

                // Image links
                assert!(images.len() > 2, "Less than 2 image links provided");
                images.iter().for_each(|img| {
                    all_links.push(img);
                    assert!(!img.contains('?'), "Passing query parameters in image link: {img}");

                });

                // Checks which should happen for all links
                for l in all_links {
                    // Must be unique
                    assert!(!seen_links.contains(l), "Duplicate link: {l}");
                    seen_links.insert(l);

                    // Must be clean
                    let clean = cleaner.clear_single_url_str(l).expect("failed cleaning link: {l}");
                    assert!(
                        matches!(&clean, std::borrow::Cow::Owned(s) if s == l), "Not a clean URL: {l}\nReplace with {clean}"
                    );
                }
            },
        )
    }
}
