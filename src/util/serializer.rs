use std::{fs::File, io::BufWriter, mem::MaybeUninit};

use rkyv::{
    rancor::{BoxedError, Panic, ResultExt, Strategy},
    ser::{allocator::SubAllocator, writer::IoWriter, Writer, WriterExt},
    SerializeUnsized,
};

use crate::{
    model::object::{ArchivableSimulateObject, SimulateAttributes},
    MAX_ALIGN, SEPARATOR,
};

type Inner<'a> = rkyv::ser::Serializer<IoWriter<BufWriter<File>>, SubAllocator<'a>, ()>;

pub struct Serializer<'a> {
    inner: Inner<'a>,
    pub osu: usize,
    pub taiko: usize,
    pub catch: usize,
    pub mania: usize,
}

impl<'a> Serializer<'a> {
    pub fn new(file: File, alloc: &'a mut [MaybeUninit<u8>]) -> Self {
        Self {
            inner: rkyv::ser::Serializer::new(
                IoWriter::new(BufWriter::new(file)),
                SubAllocator::new(alloc),
                (),
            ),
            osu: 0,
            taiko: 0,
            catch: 0,
            mania: 0,
        }
    }

    pub fn increment_mode(&mut self, obj: &ArchivableSimulateObject) {
        match obj.attrs {
            SimulateAttributes::Osu { .. } => self.osu += 1,
            SimulateAttributes::Taiko { .. } => self.taiko += 1,
            SimulateAttributes::Catch { .. } => self.catch += 1,
            SimulateAttributes::Mania { .. } => self.mania += 1,
        }
    }

    #[track_caller]
    pub fn serialize<T>(&mut self, value: &T)
    where
        T: SerializeUnsized<Strategy<Inner<'a>, BoxedError>>,
    {
        if let Err(err) = value.serialize_unsized(Strategy::<_, BoxedError>::wrap(&mut self.inner))
        {
            return println!("Failed to write entry: {err}");
        }

        WriterExt::<Panic>::pad(&mut self.inner, MAX_ALIGN - SEPARATOR.len()).always_ok();

        if let Err(err) = Writer::<BoxedError>::write(&mut self.inner, SEPARATOR) {
            println!("Failed to write separator: {err}");
        }
    }
}
