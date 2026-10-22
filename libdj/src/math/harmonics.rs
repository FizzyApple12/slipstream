use rkyv::{Archive, Deserialize, Serialize};

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Archive, Deserialize, Serialize)]
pub struct Key(i32);

impl Key {
    #[allow(clippy::return_self_not_must_use)]
    pub fn shift(self, semitones: i32) -> Self {
        let mode_offset = (self.0 / 12) * 12;
        let pitch_class = (self.0 % 12 + semitones).rem_euclid(12);

        Key(mode_offset + pitch_class)
    }

    pub fn from_semitones(semitones: i32) -> Self {
        Key(semitones.rem_euclid(24))
    }

    pub fn to_semitones(self) -> i32 {
        self.0
    }

    pub fn try_from_camelot(key: &str) -> Option<Self> {
        match key.to_lowercase().as_str() {
            "5a" => Some(Self(0)),
            "12a" => Some(Self(1)),
            "7a" => Some(Self(2)),
            "2a" => Some(Self(3)),
            "9a" => Some(Self(4)),
            "4a" => Some(Self(5)),
            "11a" => Some(Self(6)),
            "6a" => Some(Self(7)),
            "1a" => Some(Self(8)),
            "8a" => Some(Self(9)),
            "3a" => Some(Self(10)),
            "10a" => Some(Self(11)),
            "8b" => Some(Self(12)),
            "3b" => Some(Self(13)),
            "10b" => Some(Self(14)),
            "5b" => Some(Self(15)),
            "12b" => Some(Self(16)),
            "7b" => Some(Self(17)),
            "2b" => Some(Self(18)),
            "9b" => Some(Self(19)),
            "4b" => Some(Self(20)),
            "11b" => Some(Self(21)),
            "6b" => Some(Self(22)),
            "1b" => Some(Self(23)),
            _ => None,
        }
    }

    pub fn to_camelot(self) -> &'static str {
        match self.0 {
            0 => "5A",
            1 => "12A",
            2 => "7A",
            3 => "2A",
            4 => "9A",
            5 => "4A",
            6 => "11A",
            7 => "6A",
            8 => "1A",
            9 => "8A",
            10 => "3A",
            11 => "10A",
            12 => "8B",
            13 => "3B",
            14 => "10B",
            15 => "5B",
            16 => "12B",
            17 => "7B",
            18 => "2B",
            19 => "9B",
            20 => "4B",
            21 => "11B",
            22 => "6B",
            23 => "1B",
            _ => "--",
        }
    }

    pub fn try_from_traditional(key: &str) -> Option<Self> {
        match key.to_lowercase().as_str() {
            "cm" => Some(Self(0)),
            "dbm" | "c#m" | "csm" => Some(Self(1)),
            "dm" => Some(Self(2)),
            "ebm" | "d#m" | "dsm" => Some(Self(3)),
            "em" => Some(Self(4)),
            "fm" => Some(Self(5)),
            "gbm" | "f#m" | "fsm" => Some(Self(6)),
            "gm" => Some(Self(7)),
            "abm" | "g#m" | "gsm" => Some(Self(8)),
            "am" => Some(Self(9)),
            "bbm" | "a#m" | "asm" => Some(Self(10)),
            "bm" => Some(Self(11)),
            "c" => Some(Self(12)),
            "db" | "c#" | "cs" => Some(Self(13)),
            "d" => Some(Self(14)),
            "eb" | "d#" | "ds" => Some(Self(15)),
            "e" => Some(Self(16)),
            "f" => Some(Self(17)),
            "gb" | "f#" | "fs" => Some(Self(18)),
            "g" => Some(Self(19)),
            "ab" | "g#" | "gs" => Some(Self(20)),
            "a" => Some(Self(21)),
            "bb" | "a#" | "as" => Some(Self(22)),
            "b" => Some(Self(23)),
            _ => None,
        }
    }

    pub fn to_traditional(self) -> &'static str {
        match self.0 {
            0 => "Cm",
            1 => "Dbm",
            2 => "Dm",
            3 => "Ebm",
            4 => "Em",
            5 => "Fm",
            6 => "Gbm",
            7 => "Gm",
            8 => "Abm",
            9 => "Am",
            10 => "Bbm",
            11 => "Bm",
            12 => "C",
            13 => "Db",
            14 => "D",
            15 => "Eb",
            16 => "E",
            17 => "F",
            18 => "Gb",
            19 => "G",
            20 => "Ab",
            21 => "A",
            22 => "Bb",
            23 => "B",
            _ => "--",
        }
    }

    pub fn try_from(key: &str) -> Option<Self> {
        match key.to_lowercase().as_str() {
            "5a" | "cm" => Some(Self(0)),
            "12a" | "dbm" | "c#m" | "csm" => Some(Self(1)),
            "7a" | "dm" => Some(Self(2)),
            "2a" | "ebm" | "d#m" | "dsm" => Some(Self(3)),
            "9a" | "em" => Some(Self(4)),
            "4a" | "fm" => Some(Self(5)),
            "11a" | "gbm" | "f#m" | "fsm" => Some(Self(6)),
            "6a" | "gm" => Some(Self(7)),
            "1a" | "abm" | "g#m" | "gsm" => Some(Self(8)),
            "8a" | "am" => Some(Self(9)),
            "3a" | "bbm" | "a#m" | "asm" => Some(Self(10)),
            "10a" | "bm" => Some(Self(11)),
            "8b" | "c" => Some(Self(12)),
            "3b" | "db" | "c#" | "cs" => Some(Self(13)),
            "10b" | "d" => Some(Self(14)),
            "5b" | "eb" | "d#" | "ds" => Some(Self(15)),
            "12b" | "e" => Some(Self(16)),
            "7b" | "f" => Some(Self(17)),
            "2b" | "gb" | "f#" | "fs" => Some(Self(18)),
            "9b" | "g" => Some(Self(19)),
            "4b" | "ab" | "g#" | "gs" => Some(Self(20)),
            "11b" | "a" => Some(Self(21)),
            "6b" | "bb" | "a#" | "as" => Some(Self(22)),
            "1b" | "b" => Some(Self(23)),
            _ => None,
        }
    }
}
