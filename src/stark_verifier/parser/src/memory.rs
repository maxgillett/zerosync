#[derive(Copy, Clone)]
enum MemoryEntryType {
    Pointer,
    Value,
}

#[derive(Copy, Clone)]
pub struct MemoryEntry {
    entry_type: MemoryEntryType,
    value: u64,
}

impl MemoryEntry {
    pub fn new_pointer(value: usize) -> MemoryEntry {
        MemoryEntry {
            entry_type: MemoryEntryType::Pointer,
            value: value as u64,
        }
    }

    pub fn new_value(value: u64) -> MemoryEntry {
        MemoryEntry {
            entry_type: MemoryEntryType::Value,
            value,
        }
    }

    pub fn to_hex(&self) -> String {
        format!("{:#X}", self.value)
    }

    pub fn make_absolute(&self, pointers: &Vec<usize>) -> String {
        let pointer = pointers[self.value as usize];
        format!("{}", pointer)
    }
}

type Memory = Vec<MemoryEntry>;

pub struct DynamicMemory<'a> {
    memories: &'a mut Vec<Memory>,
    segment: usize,
}

impl<'a> DynamicMemory<'a> {
    pub fn new(memories: &'a mut Vec<Memory>) -> DynamicMemory<'a> {
        memories.push(Vec::<MemoryEntry>::new());
        DynamicMemory {
            memories: memories,
            segment: 0,
        }
    }

    pub fn serialize(&self) -> Vec<String> {
        // Concatenate all temporary memories and compute absolute pointers
        let mut concatenated = Vec::<MemoryEntry>::new();
        let mut pointers = Vec::new();

        for vector in &mut self.memories.iter() {
            pointers.push(concatenated.len());
            concatenated.extend(vector);
        }

        // Make the relative pointers absolute
        let mut memory = Vec::new();
        for entry in concatenated {
            match entry.entry_type {
                MemoryEntryType::Pointer => {
                    memory.push(entry.make_absolute(&pointers));
                }
                MemoryEntryType::Value => {
                    memory.push(entry.to_hex());
                }
            }
        }

        memory
    }

    fn write_entry(&mut self, entry: MemoryEntry) {
        self.memories.get_mut(self.segment).unwrap().push(entry);
    }

    pub fn write_pointer(&mut self, pointer: usize) {
        self.write_entry(MemoryEntry::new_pointer(pointer))
    }

    pub fn write_value(&mut self, value: u64) {
        self.write_entry(MemoryEntry::new_value(value))
    }

    pub fn write_array<T: Writeable>(&mut self, array: Vec<T>) {
        let mut sub_memory = self.alloc();
        for writable in array {
            writable.write_into(&mut sub_memory);
        }
    }

    pub fn write_array_with<Q, T: WriteableWith<Q>, F>(&mut self, array: Vec<T>, f: F)
    where
        F: Fn(u32) -> Q,
    {
        let mut sub_memory = self.alloc();
        let mut i = 0;
        for writable in array {
            writable.write_into(&mut sub_memory, f(i));
            i += 1;
        }
    }

    fn alloc(&mut self) -> DynamicMemory {
        let segment = self.memories.len();
        self.write_pointer(segment);
        self.memories.push(Vec::<MemoryEntry>::new());
        DynamicMemory {
            memories: self.memories,
            segment: segment,
        }
    }
}

pub trait Writeable {
    fn write_into(&self, target: &mut DynamicMemory);
}

impl Writeable for u8 {
    fn write_into(&self, target: &mut DynamicMemory) {
        target.write_value(*self as u64)
    }
}

impl Writeable for u32 {
    fn write_into(&self, target: &mut DynamicMemory) {
        target.write_value(*self as u64)
    }
}

impl Writeable for u64 {
    fn write_into(&self, target: &mut DynamicMemory) {
        target.write_value(*self)
    }
}

impl Writeable for usize {
    fn write_into(&self, target: &mut DynamicMemory) {
        target.write_value(*self as u64)
    }
}

pub trait WriteableWith<Parameters> {
    fn write_into(&self, target: &mut DynamicMemory, params: Parameters);
}
