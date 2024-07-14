use garde::Validate;
use once_cell::sync::Lazy;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use strum_macros::EnumIter;

/// Human history time period of a world wonder
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, JsonSchema)]
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
            i16::MIN..=-3000 => TimePeriod::Prehistoric,
            -2999..=-800 => TimePeriod::Ancient,
            -799..=500 => TimePeriod::Classical,
            501..=1500 => TimePeriod::PostClassical,
            1501..=1800 => TimePeriod::EarlyModern,
            1801..=i16::MAX => TimePeriod::Modern,
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
    use chrono::prelude::*;
    use std::collections::HashSet;

    use super::*;

    #[test]
    fn validate_wonders_data() {
        assert!(WONDERS.len() > 0);

        // Validate data using `garde`
        WONDERS.iter().for_each(|w| w.validate(&()).unwrap());

        // To check for duplicate names
        let mut seen_names = HashSet::new();
        // To check for duplicate links
        let mut seen_links = HashSet::new();

        // Current year
        let year = Utc::now().year();

        WONDERS.iter().for_each(
            |Wonder {
                name,
                location,
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
                assert!(!name.trim().is_empty(), "Name provided is empty");
                assert!(!location.trim().is_empty(), "Location provided is empty");
                assert_eq!(name.trim().len(), name.len(), "Name contains trailing/leading whitespace");
                assert_eq!(location.trim().len(), location.len(), "Location contains trailing/leading whitespace");

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
                // Wiki link
                assert!(!seen_links.contains(wiki), "Duplicate link: {wiki}");
                seen_links.insert(wiki);

                // Other links (`Option` values)
                [britannica, google_maps, trip_advisor].into_iter().for_each(|l| {
                    if let Some(l) = l {
                        assert!(!seen_links.contains(l), "Duplicate link: {l}");
                        seen_links.insert(l);
                    };
                });

                // Image links
                assert!(images.len() > 2, "Less than 2 image links provided");
                images.iter().for_each(|img| {
                    assert!(!seen_links.contains(img), "Duplicate link: {img}");
                    seen_links.insert(img);
                });

                assert!(!seen_names.contains(name.as_str()), "Duplicate name: '{name}'");
                seen_names.insert(name.as_str());
            },
        )
    }
}
