use serde::{Deserialize, Serialize};
use std::fmt::Display;

/// A trade with the position (long/short/out) and number of shares commit to
/// the trade.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Position {
    /// Buy N shares.
    Long(u64),
    /// Short N shares.
    Short(u64),
    /// Close position and hold nothing.
    Out,
    /// Hold current position.
    Hold,
}

impl Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Position::Long(shares) => write!(f, "Long({})", shares),
            Position::Short(shares) => write!(f, "Short({})", shares),
            Position::Out => write!(f, "Out"),
            Position::Hold => write!(f, "Hold"),
        }
    }
}

impl Position {
    /// True if the position is a long or short position.
    pub fn is_entry(&self) -> bool {
        match self {
            Position::Long(_) => true,
            Position::Short(_) => true,
            Position::Out => false,
            Position::Hold => false,
        }
    }

    /// True if the `other` position is an exit from the current position.
    /// e.g. going short when you are currently long implies that you sell the
    // shares. Going out from any entry position (long/short) is an exit.
    pub fn is_exit_from(&self, other: Position) -> bool {
        // TODO: make sure to test this at some point
        match (self, other) {
            (Position::Out, p) if p.is_entry() => true,
            (Position::Long(_), Position::Short(_)) => true,
            (Position::Short(_), Position::Long(_)) => true,
            // (Position::Out, Position::Long(_)) => true,
            // (Position::Out, Position::Short(_)) => true,
            // (Position::Long(_),  Position::Out) => true,
            // (Position::Short(_), Position::Out) => true,
            _ => false,
        }
    }

    /// Long and short are opposite, all others are not.
    pub fn is_opposite(&self, other: Position) -> bool {
        match (&self, other) {
            (Position::Long(_), Position::Short(_)) => true,
            (Position::Short(_), Position::Long(_)) => true,
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Position;

    #[test]
    fn entry_and_exit() {
        let long = Position::Long(1);
        let short = Position::Short(1);
        let out = Position::Out;
        let hold = Position::Hold;

        assert!(long.is_entry());
        assert!(long.is_entry());
        assert!(!out.is_entry());
        assert!(!hold.is_entry());

        assert!(out.is_exit_from(long));
        assert!(out.is_exit_from(short));
        assert!(!long.is_exit_from(out));
        assert!(!short.is_exit_from(out));

        assert!(long.is_exit_from(short));
        assert!(short.is_exit_from(long));

        assert!(!long.is_exit_from(long));
        assert!(!short.is_exit_from(short));
    }
}
