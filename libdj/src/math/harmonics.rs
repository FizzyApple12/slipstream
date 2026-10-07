use rkyv::{Archive, Deserialize, Serialize};

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Archive, Deserialize, Serialize)]
pub struct Key(i32);

impl Key {
    #[allow(clippy::return_self_not_must_use)]
    pub fn shift(self, semitones: i32) -> Self {
        if semitones >= 0 {
            Key((self.0 + semitones) % 24)
        } else {
            Key(23 - (((23 - self.0) + -semitones) % 24))
        }
    }

    pub fn from_semitones(semitones: i32) -> Self {
        if semitones >= 0 {
            Key((semitones) % 24)
        } else {
            Key(23 - ((-semitones) % 24))
        }
    }

    pub fn try_from_camelot(key: &str) -> Option<Self> {
        match key.to_lowercase().as_str() {
            "1a" => Some(Self(0)),
            "1b" => Some(Self(1)),
            "2a" => Some(Self(2)),
            "2b" => Some(Self(3)),
            "3a" => Some(Self(4)),
            "3b" => Some(Self(5)),
            "4a" => Some(Self(6)),
            "4b" => Some(Self(7)),
            "5a" => Some(Self(8)),
            "5b" => Some(Self(9)),
            "6a" => Some(Self(10)),
            "6b" => Some(Self(11)),
            "7a" => Some(Self(12)),
            "7b" => Some(Self(13)),
            "8a" => Some(Self(14)),
            "8b" => Some(Self(15)),
            "9a" => Some(Self(16)),
            "9b" => Some(Self(17)),
            "10a" => Some(Self(18)),
            "10b" => Some(Self(19)),
            "11a" => Some(Self(20)),
            "11b" => Some(Self(21)),
            "12a" => Some(Self(22)),
            "12b" => Some(Self(23)),
            _ => None,
        }
    }

    pub fn to_camelot(self) -> &'static str {
        match self.0 {
            0 => "1A",
            1 => "1B",
            2 => "2A",
            3 => "2B",
            4 => "3A",
            5 => "3B",
            6 => "4A",
            7 => "4B",
            8 => "5A",
            9 => "5B",
            10 => "6A",
            11 => "6B",
            12 => "7A",
            13 => "7B",
            14 => "8A",
            15 => "8B",
            16 => "9A",
            17 => "9B",
            18 => "10A",
            19 => "10B",
            20 => "11A",
            21 => "11B",
            22 => "12A",
            23 => "12B",
            _ => "--",
        }
    }

    pub fn try_from_traditional(key: &str) -> Option<Self> {
        match key.to_lowercase().as_str() {
            "abm" | "g#m" | "gsm" => Some(Self(0)),
            "b" => Some(Self(1)),
            "ebm" | "d#m" | "dsm" => Some(Self(2)),
            "gb" | "f#" | "fs" => Some(Self(3)),
            "bbm" | "a#m" | "asm" => Some(Self(4)),
            "db" | "c#" | "cs" => Some(Self(5)),
            "fm" => Some(Self(6)),
            "ab" | "g#" | "gs" => Some(Self(7)),
            "cm" => Some(Self(8)),
            "eb" | "d#" | "ds" => Some(Self(9)),
            "gm" => Some(Self(10)),
            "bb" | "a#" | "as" => Some(Self(11)),
            "dm" => Some(Self(12)),
            "f" => Some(Self(13)),
            "am" => Some(Self(14)),
            "c" => Some(Self(15)),
            "em" => Some(Self(16)),
            "g" => Some(Self(17)),
            "bm" => Some(Self(18)),
            "d" => Some(Self(19)),
            "gbm" | "f#m" | "fsm" => Some(Self(20)),
            "a" => Some(Self(21)),
            "dbm" | "c#m" | "csm" => Some(Self(22)),
            "e" => Some(Self(23)),
            _ => None,
        }
    }

    pub fn to_traditional(self) -> &'static str {
        match self.0 {
            0 => "Abm",
            1 => "B",
            2 => "Ebm",
            3 => "Gb",
            4 => "Bbm",
            5 => "Db",
            6 => "Fm",
            7 => "Ab",
            8 => "Cm",
            9 => "Eb",
            10 => "Gm",
            11 => "Bb",
            12 => "Dm",
            13 => "F",
            14 => "Am",
            15 => "C",
            16 => "Em",
            17 => "G",
            18 => "Bm",
            19 => "D",
            20 => "Gbm",
            21 => "A",
            22 => "Dbm",
            23 => "E",
            _ => "--",
        }
    }

    pub fn try_from(key: &str) -> Option<Self> {
        match key.to_lowercase().as_str() {
            "1a" | "abm" | "g#m" | "gsm" => Some(Self(0)),
            "1b" | "b" => Some(Self(1)),
            "2a" | "ebm" | "d#m" | "dsm" => Some(Self(2)),
            "2b" | "gb" | "f#" | "fs" => Some(Self(3)),
            "3a" | "bbm" | "a#m" | "asm" => Some(Self(4)),
            "3b" | "db" | "c#" | "cs" => Some(Self(5)),
            "4a" | "fm" => Some(Self(6)),
            "4b" | "ab" | "g#" | "gs" => Some(Self(7)),
            "5a" | "cm" => Some(Self(8)),
            "5b" | "eb" | "d#" | "ds" => Some(Self(9)),
            "6a" | "gm" => Some(Self(10)),
            "6b" | "bb" | "a#" | "as" => Some(Self(11)),
            "7a" | "dm" => Some(Self(12)),
            "7b" | "f" => Some(Self(13)),
            "8a" | "am" => Some(Self(14)),
            "8b" | "c" => Some(Self(15)),
            "9a" | "em" => Some(Self(16)),
            "9b" | "g" => Some(Self(17)),
            "10a" | "bm" => Some(Self(18)),
            "10b" | "d" => Some(Self(19)),
            "11a" | "gbm" | "f#m" | "fsm" => Some(Self(20)),
            "11b" | "a" => Some(Self(21)),
            "12a" | "dbm" | "c#m" | "csm" => Some(Self(22)),
            "12b" | "e" => Some(Self(23)),
            _ => None,
        }
    }
}
