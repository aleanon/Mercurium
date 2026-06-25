use std::fmt::Display;

#[derive(Debug, Clone)]
pub enum AccountSummary {
    NoUpdateReceived,
    NoLedgerPresense,
    Summary {
        nr_of_fungibles: usize,
        nr_of_non_fungibles: usize,
    },
}

impl AccountSummary {
    pub fn has_summary(&self) -> bool {
        matches!(self, Self::Summary { .. })
    }
}

impl Display for AccountSummary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoUpdateReceived => write!(f, "No update"),
            Self::NoLedgerPresense => write!(f, "None"),
            Self::Summary {
                nr_of_fungibles,
                nr_of_non_fungibles,
            } => write!(f, "{} Assets", nr_of_fungibles + nr_of_non_fungibles),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn has_summary_only_for_summary_variant() {
        assert!(AccountSummary::Summary { nr_of_fungibles: 1, nr_of_non_fungibles: 2 }.has_summary());
        assert!(!AccountSummary::NoUpdateReceived.has_summary());
        assert!(!AccountSummary::NoLedgerPresense.has_summary());
    }

    #[test]
    fn display_formats_each_variant() {
        assert_eq!(format!("{}", AccountSummary::NoUpdateReceived), "No update");
        assert_eq!(format!("{}", AccountSummary::NoLedgerPresense), "None");
        assert_eq!(
            format!("{}", AccountSummary::Summary { nr_of_fungibles: 3, nr_of_non_fungibles: 4 }),
            "7 Assets"
        );
    }
}
