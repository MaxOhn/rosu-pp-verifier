use std::fmt;

#[derive(Default, serde::Deserialize, rkyv::Archive, rkyv::Serialize)]
#[serde(default)]
#[rkyv(compare(PartialEq))]
pub struct Statistics {
    pub perfect: u32,
    pub great: u32,
    pub good: u32,
    pub ok: u32,
    pub meh: u32,
    pub miss: u32,
    pub large_tick_hit: u32,
    pub small_tick_hit: u32,
    pub slider_tail_hit: u32,
}

impl Statistics {
    pub fn is_eq(&self, other: &ArchivedStatistics, mode: u8, lazer: bool) -> bool {
        if mode != 0 || lazer {
            return self == other;
        }

        self.great == other.great
            && self.ok == other.ok
            && self.meh == other.meh
            && self.miss == other.miss
    }
}

impl fmt::Debug for Statistics {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut debug = f.debug_struct("Statistics");

        if self.perfect > 0 {
            debug.field("perfect", &self.perfect);
        }

        if self.great > 0 {
            debug.field("great", &self.great);
        }

        if self.good > 0 {
            debug.field("good", &self.good);
        }

        if self.ok > 0 {
            debug.field("ok", &self.ok);
        }

        if self.meh > 0 {
            debug.field("meh", &self.meh);
        }

        if self.miss > 0 {
            debug.field("miss", &self.miss);
        }

        if self.large_tick_hit > 0 {
            debug.field("large_tick_hit", &self.large_tick_hit);
        }

        if self.small_tick_hit > 0 {
            debug.field("small_tick_hit", &self.small_tick_hit);
        }

        if self.slider_tail_hit > 0 {
            debug.field("slider_tail_hit", &self.slider_tail_hit);
        }

        debug.finish()
    }
}

impl fmt::Debug for ArchivedStatistics {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut debug = f.debug_struct("Statistics");

        if self.perfect > 0 {
            debug.field("perfect", &self.perfect);
        }

        if self.great > 0 {
            debug.field("great", &self.great);
        }

        if self.good > 0 {
            debug.field("good", &self.good);
        }

        if self.ok > 0 {
            debug.field("ok", &self.ok);
        }

        if self.meh > 0 {
            debug.field("meh", &self.meh);
        }

        if self.miss > 0 {
            debug.field("miss", &self.miss);
        }

        if self.large_tick_hit > 0 {
            debug.field("large_tick_hit", &self.large_tick_hit);
        }

        if self.small_tick_hit > 0 {
            debug.field("small_tick_hit", &self.small_tick_hit);
        }

        if self.slider_tail_hit > 0 {
            debug.field("slider_tail_hit", &self.slider_tail_hit);
        }

        debug.finish()
    }
}
