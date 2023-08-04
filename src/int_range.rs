use anyhow::{bail, Context};

#[derive(PartialEq, Clone)]
pub(crate) struct IntRange {
    start: Option<u32>,
    end: Option<u32>,
    end_inclusive: bool,
}

#[allow(dead_code)]
impl IntRange {
    /// Checks to see if the given value is within the given range and returns true if it is.
    fn contains(&self, value: u32) -> bool {
        match (self.start, self.end, self.end_inclusive) {
            (None, None, _) => true,
            (None, Some(end), true) => value <= end,
            (None, Some(end), false) => value < end,
            (Some(start), None, _) => value >= start,
            (Some(start), Some(end), true) => value >= start && value <= end,
            (Some(start), Some(end), false) => value >= start && value < end,
        }
    }

    fn new(start: Option<u32>, end: Option<u32>, end_inclusive: bool) -> Self {
        Self {
            start,
            end,
            end_inclusive,
        }
    }

    fn new_range(start: u32, end: u32) -> Self {
        Self {
            start: Some(start),
            end: Some(end),
            end_inclusive: false,
        }
    }

    fn new_inclusive_range(start: u32, end: u32) -> Self {
        Self {
            start: Some(start),
            end: Some(end),
            end_inclusive: true,
        }
    }
}

impl std::fmt::Debug for IntRange {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}..{:?}{}",
            if let Some(start) = self.start {
                start.to_string()
            } else {
                "".to_string()
            },
            if self.end_inclusive {
                "=".to_string()
            } else {
                "".to_string()
            },
            if let Some(end) = self.start {
                end.to_string()
            } else {
                "".to_string()
            },
        )?;
        Ok(())
    }
}

impl std::str::FromStr for IntRange {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let mut parts = s.split("..");
        let start = match parts.next() {
            Some(start_str) => {
                if start_str.is_empty() {
                    None
                } else {
                    Some(start_str.parse::<u32>().context("Invalid start value")?)
                }
            }
            None => bail!("Invalid range syntax"),
        };

        let end_and_inclusive = match parts.next() {
            Some(end_str) => {
                if end_str.starts_with('=') {
                    (end_str.trim_start_matches('='), true)
                } else {
                    (end_str, false)
                }
            }
            None => {
                return Ok(Self {
                    start,
                    end: start,
                    end_inclusive: false,
                })
            }
        };

        let end = match end_and_inclusive.0 {
            "" => None,
            _ => Some(
                end_and_inclusive
                    .0
                    .parse::<u32>()
                    .context("Invalid end value")?,
            ),
        };

        Ok(IntRange {
            start,
            end,
            end_inclusive: end_and_inclusive.1,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn parse_valid_range() -> anyhow::Result<()> {
        assert_eq!(IntRange::from_str("5..10")?, IntRange::new_range(5, 10));
        assert_eq!(IntRange::from_str("1..1")?, IntRange::new_range(1, 1));
        Ok(())
    }

    #[test]
    fn parse_valid_inclusive_range() -> anyhow::Result<()> {
        assert_eq!(
            IntRange::from_str("0..=100")?,
            IntRange::new_inclusive_range(0, 100)
        );

        assert_eq!(
            IntRange::from_str("..=10")?,
            IntRange::new(None, Some(10), true)
        );
        Ok(())
    }

    #[test]
    fn parse_valid_unbound() -> anyhow::Result<()> {
        assert_eq!(IntRange::from_str("..")?, IntRange::new(None, None, false));
        Ok(())
    }

    #[test]
    fn parse_valid_unit() -> anyhow::Result<()> {
        assert_eq!(IntRange::from_str("10")?, IntRange::new_range(10, 10));
        Ok(())
    }

    #[test]
    fn contains_range_too_low() {
        assert_eq!(IntRange::new_range(5, 10).contains(4), false);
    }

    #[test]
    fn contains_range_too_high() {
        assert_eq!(IntRange::new_range(5, 10).contains(11), false);
    }

    #[test]
    fn contains_range_start_inclusive() {
        assert_eq!(IntRange::new_range(5, 10).contains(5), true);
        assert_eq!(IntRange::new(Some(5), None, false).contains(5), true);
    }

    #[test]
    fn contains_range_end_inclusive() {
        assert_eq!(IntRange::new_inclusive_range(5, 10).contains(10), true);
        assert_eq!(IntRange::new_range(5, 10).contains(10), false);
        assert_eq!(IntRange::new(None, Some(10), true).contains(10), true);
        assert_eq!(IntRange::new(None, Some(10), false).contains(10), false);
    }

    #[test]
    fn valid_range() {
        assert_eq!(IntRange::new_range(5, 10).contains(6), true);
        assert_eq!(IntRange::new(None, Some(10), false).contains(1), true);
        assert_eq!(IntRange::new(None, Some(10), true).contains(6), true);
    }

    #[test]
    fn invalid_start() {
        assert_eq!(
            IntRange::from_str("a..10").err().unwrap().to_string(),
            "Invalid start value"
        );

        assert_eq!(
            IntRange::from_str("-10..10").err().unwrap().to_string(),
            "Invalid start value"
        );
    }
    #[test]
    fn dont_allow_minus_numbers() {
        assert_eq!(
            IntRange::from_str("-10..-10").err().unwrap().to_string(),
            "Invalid start value"
        );
        assert_eq!(
            IntRange::from_str("-10..10").err().unwrap().to_string(),
            "Invalid start value"
        );
        assert_eq!(
            IntRange::from_str("10..-10").err().unwrap().to_string(),
            "Invalid end value"
        );
    }
}
