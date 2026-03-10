use rkyv::Archived;
use rosu_pp::Beatmap;

use crate::MAP_PATH;

#[derive(Default)]
pub struct RecentBeatmap {
    inner: Option<Inner>,
}

struct Inner {
    map_id: Archived<i32>,
    map: Beatmap,
}

impl RecentBeatmap {
    pub fn get(&mut self, map_id: Archived<i32>) -> Option<&Beatmap> {
        let check = self
            .inner
            .as_ref()
            .is_some_and(|inner| inner.map_id == map_id);

        if check {
            // NLL...
            return Some(&self.inner.as_ref().unwrap().map);
        }

        let path = format!("{}{map_id}.osu", &*MAP_PATH);

        match Beatmap::from_path(&path) {
            Ok(map) => Some(&self.inner.insert(Inner { map_id, map }).map),
            Err(err) => {
                println!("Failed to decode map at `{path}`: {err}");

                None
            }
        }
    }
}
