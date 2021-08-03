use std::fmt::{self, Display};

const VALID_READ: u32 = 3;
const VALID_FREQ: f32 = 0.01;
const CONF_FREQ: f32 = 0.96;

#[derive(Debug, PartialEq)]
pub enum Linkage {
    Cis,
    Trans,
    Super,
    Sub,
    Cross,
}

impl Display for Linkage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::Cis => write!(f, "cis"),
            Self::Trans => write!(f, "trans"),
            Self::Super => write!(f, "super"),
            Self::Sub => write!(f, "sub"),
            Self::Cross => write!(f, "cross"),
        }
    }
}

#[derive(Debug, Default)]
pub struct Link {
    pub both: u32,
    pub first: u32,
    pub second: u32,
    pub neither: u32,
}

impl Link {
    #[inline]
    fn either(&self) -> u32 {
        // Read count that support only first or second.
        self.first + self.second
    }

    #[inline]
    fn minor(&self) -> u32 {
        u32::min(self.first, self.second)
    }

    #[inline]
    fn major(&self) -> u32 {
        u32::max(self.first, self.second)
    }

    #[inline]
    fn any(&self) -> u32 {
        self.both + self.either()
    }

    /// Read support both freq among read support any variants.
    #[inline]
    fn both_freq_any(&self) -> f32 {
        (self.both as f32) / (self.any() as f32)
    }

    #[inline]
    fn minor_freq_any(&self) -> f32 {
        (self.minor() as f32) / (self.any() as f32)
    }

    #[inline]
    fn major_freq_any(&self) -> f32 {
        (self.major() as f32) / (self.any() as f32)
    }

    #[inline]
    fn minor_freq_both(&self) -> f32 {
        (self.minor() as f32) / ((self.minor() + self.both) as f32)
    }

    #[inline]
    fn first_freq_any(&self) -> f32 {
        (self.first as f32) / (self.any() as f32)
    }

    #[inline]
    fn second_freq_any(&self) -> f32 {
        (self.second as f32) / (self.any() as f32)
    }

    /// Infer linkage based on link.
    pub fn infer_linkage(&self) -> Option<Linkage> {
        if self.both >= VALID_READ
            && self.both_freq_any() >= CONF_FREQ
            && self.major() < VALID_READ
            && self.major_freq_any() < VALID_FREQ
        {
            Some(Linkage::Cis)
        } else if self.both < VALID_READ
            && self.both_freq_any() < VALID_FREQ
            && self.minor() >= VALID_READ
            && self.minor_freq_both() >= CONF_FREQ
        {
            Some(Linkage::Trans)
        } else if self.both >= VALID_READ
            && self.first < VALID_READ
            && self.first_freq_any() < VALID_FREQ
            && self.second >= VALID_READ
        {
            Some(Linkage::Sub)
        } else if self.both >= VALID_READ
            && self.second < VALID_READ
            && self.second_freq_any() < VALID_FREQ
            && self.first >= VALID_READ
        {
            Some(Linkage::Super)
        } else if self.both >= VALID_READ
            && self.both_freq_any() >= VALID_FREQ
            && self.minor() >= VALID_READ
            && self.minor_freq_any() >= VALID_FREQ
        {
            Some(Linkage::Cross)
        } else {
            None
        }
    }
}

impl Display for Link {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(v) = self.infer_linkage() {
            write!(
                f,
                "{{\n  \"both\": {},\n  \"first\": {},\n  \"second\": {},\n  \"neither\": {},\n  \"conclusion\": \"{}\"\n}}",
                self.both, self.first, self.second, self.neither, v
            )
        } else {
            write!(
                f,
                "{{\n  \"both\": {},\n  \"first\": {},\n  \"second\": {},\n  \"neither\": {},\n  \"conclusion\": \"undefined\"\n}}",
                self.both, self.first, self.second, self.neither
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_infer_cis() {
        let link = Link {
            both: 101,
            first: 1,
            second: 0,
            neither: 100,
        };
        assert_eq!(link.infer_linkage(), Some(Linkage::Cis));
    }

    #[test]
    fn test_infer_trans() {
        let link = Link {
            both: 1,
            first: 50,
            second: 70,
            neither: 100,
        };
        assert_eq!(link.infer_linkage(), Some(Linkage::Trans));
    }

    #[test]
    fn test_infer_super() {
        let link = Link {
            both: 28,
            first: 50,
            second: 0,
            neither: 100,
        };
        assert_eq!(link.infer_linkage(), Some(Linkage::Super));
    }

    #[test]
    fn test_infer_sub() {
        let link = Link {
            both: 58,
            first: 1,
            second: 50,
            neither: 100,
        };
        assert_eq!(link.infer_linkage(), Some(Linkage::Sub));
    }
}
