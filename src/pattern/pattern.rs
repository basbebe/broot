
use {
    super::{
        FuzzyPattern,
        RegexPattern,
    },
    crate::{
        app::AppContext,
        command::PatternParts,
        errors::PatternError,
    },
    std::{
        fmt,
        mem,
    },
};

/// a pattern for filtering and sorting filenames.
/// It's backed either by a fuzzy pattern matcher or
///  by a regular expression (in which case there's no real
///  score)
#[derive(Debug, Clone)]
pub enum Pattern {
    None,
    NameFuzzy(FuzzyPattern),
    PathFuzzy(FuzzyPattern),
    NameRegex(RegexPattern),
    //PathRegex(RegexPattern),
}

impl fmt::Display for Pattern {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Pattern::NameFuzzy(fp) => write!(f, "NameFuzzy({})", fp),
            Pattern::PathFuzzy(fp) => write!(f, "PathFuzzy({})", fp),
            Pattern::NameRegex(rp) => write!(f, "NameRegex({})", rp),
            //Pattern::PathRegex(rp) => write!(f, "PathRegex({})", rp),
            Pattern::None => write!(f, "None"),
        }
    }
}

impl Pattern {
    pub fn from_parts(
        parts: &PatternParts,
        _con: &AppContext,
    ) -> Result<Pattern, PatternError> {
        match parts.mode.as_deref() {
            None | Some("f") => Ok(Self::name_fuzzy(&parts.pattern)),
            Some("p") => Ok(Self::path_fuzzy(&parts.pattern)),
            Some("r") | Some("") => Self::regex(
                &parts.pattern,
                parts.flags.as_deref().unwrap_or(""),
            ),
            Some(mode) => Err(PatternError::InvalidMode { mode: mode.to_string() }),
        }
    }
    /// create a new fuzzy pattern
    pub fn name_fuzzy(pat: &str) -> Pattern {
        if pat.is_empty() {
            Pattern::None
        } else {
            Pattern::NameFuzzy(FuzzyPattern::from(pat))
        }
    }
    /// create a new fuzzy pattern
    pub fn path_fuzzy(pat: &str) -> Pattern {
        if pat.is_empty() {
            Pattern::None
        } else {
            Pattern::PathFuzzy(FuzzyPattern::from(pat))
        }
    }
    /// try to create a regex pattern
    pub fn regex(pat: &str, flags: &str) -> Result<Pattern, PatternError> {
        Ok(if pat.is_empty() {
            Pattern::None
        } else {
            Pattern::NameRegex(RegexPattern::from(pat, flags)?)
        })
    }
    pub fn applies_to_path(&self) -> bool {
        match self {
            Self::PathFuzzy(_) => true,
            _ => false,
        }
    }
    pub fn find(&self, candidate: &str) -> Option<Match> {
        match self {
            Self::NameFuzzy(fp) => fp.find(candidate),
            Self::PathFuzzy(fp) => fp.find(candidate),
            Self::NameRegex(rp) => rp.find(candidate),
            Self::None => Some(Match {
                // this isn't really supposed to be used
                score: 1,
                pos: Vec::with_capacity(0),
            }),
        }
    }
    /// compute the score of a string
    /// Caller is responsible of calling applies_to_path
    /// and preparing and providing the relevant string.
    pub fn score_of(&self, candidate: &str) -> Option<i32> {
        match self {
            Pattern::NameFuzzy(fp) => fp.score_of(candidate),
            Pattern::PathFuzzy(fp) => fp.score_of(candidate),
            Pattern::NameRegex(rp) => rp.find(candidate).map(|m| m.score),
            Pattern::None => None,
        }
    }
    pub fn is_some(&self) -> bool {
        match self {
            Pattern::None => false,
            _ => true,
        }
    }
    pub fn as_input(&self) -> String {
        match self {
            Pattern::NameFuzzy(fp) => fp.to_string(),
            Pattern::PathFuzzy(fp) => format!("p/{}", fp),
            Pattern::NameRegex(rp) => format!("rp/{}", rp),
            Pattern::None => String::new(),
        }
    }
    /// empties the pattern and return it
    /// Similar to Option::take
    pub fn take(&mut self) -> Pattern {
        mem::replace(self, Pattern::None)
    }
    /// return the number of results we should find before starting to
    ///  sort them (unless time is runing out).
    pub fn optimal_result_number(&self, targeted_size: usize) -> usize {
        match self {
            Pattern::NameFuzzy(fp) => fp.optimal_result_number(targeted_size),
            Pattern::PathFuzzy(fp) => fp.optimal_result_number(targeted_size),
            Pattern::NameRegex(rp) => rp.optimal_result_number(targeted_size),
            Pattern::None => targeted_size,
        }
    }
}

/// A Match is a positive result of pattern matching
#[derive(Debug, Clone)]
pub struct Match {
    pub score: i32, // score of the match, guaranteed strictly positive, bigger is better
    pub pos: Vec<usize>, // positions of the matching chars
}
