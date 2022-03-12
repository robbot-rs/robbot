use robbot::store::{
    DataDescriptor, DataQuery, Deserialize, Deserializer, Serialize, Serializer, Store, StoreData,
    TypeSerializer,
};

use async_trait::async_trait;
use parking_lot::RwLock;

use std::collections::HashMap;
use std::convert::Infallible;
use std::mem;
use std::ptr;
use std::slice;
use std::sync::Arc;

/// An efficient [`Store`] that keeps all entries in memory.
#[derive(Clone, Debug, Default)]
pub struct MemStore {
    // inner: Arc<RwLock<HashMap<String, Vec<Vec<u8>>>>>,
    inner: Arc<RwLock<HashMap<String, Vec<Entry>>>>,
}

#[derive(Debug)]
struct Entry {
    /// The memory buffer, all pointers in keys must point into buf.
    buf: Vec<u8>,
    keys: HashMap<String, *const u8>,
}

impl Entry {
    /// Compare this entry with another. If the key provided by `other` doesn't exist
    /// on this entry, `None` is returned.
    fn eq(&self, key: &str, other: TypePtr) -> Option<bool> {
        let left_ptr = *self.keys.get(key)?;
        let right_ptr = other.ptr;

        unsafe {
            Some(match other._type {
                StoreType::Bool | StoreType::I8 | StoreType::U8 => {
                    let left = *left_ptr;
                    let right = *right_ptr;

                    left == right
                }
                StoreType::I16 | StoreType::U16 => {
                    let left = slice::from_raw_parts(left_ptr, 2);
                    let right = slice::from_raw_parts(right_ptr, 2);

                    left == right
                }
                StoreType::I32 | StoreType::U32 | StoreType::F32 => {
                    let left = slice::from_raw_parts(left_ptr, 4);
                    let right = slice::from_raw_parts(right_ptr, 4);

                    left == right
                }
                StoreType::I64 | StoreType::U64 | StoreType::F64 => {
                    let left = slice::from_raw_parts(left_ptr, 8);
                    let right = slice::from_raw_parts(right_ptr, 8);

                    left == right
                }
                StoreType::String => {
                    let left = {
                        let len = slice::from_raw_parts(left_ptr, mem::size_of::<usize>());
                        let len: [u8; mem::size_of::<usize>()] = mem::transmute_copy(&len[0]);
                        let len = usize::from_ne_bytes(len);

                        // Read the whole string including the prefixed length.
                        slice::from_raw_parts(left_ptr, mem::size_of::<usize>() + len)
                    };

                    let right = {
                        let len = slice::from_raw_parts(right_ptr, mem::size_of::<usize>());
                        let len: [u8; mem::size_of::<usize>()] = mem::transmute_copy(&len[0]);
                        let len = usize::from_ne_bytes(len);

                        slice::from_raw_parts(right_ptr, mem::size_of::<usize>() + len)
                    };

                    left == right
                }
            })
        }
    }

    /// Copy the entry buffer to create type `T`.
    unsafe fn copy_into<T>(&self) -> T
    where
        T: StoreData<MemStore>,
    {
        let mut deserializer = MemDeserializer::new(self.buf.as_ptr());
        T::deserialize(&mut deserializer).unwrap()
    }
}

// Entry is Send and Sync because all *const u8 pointers in `keys` point into
// `buf`.
unsafe impl Send for Entry {}
unsafe impl Sync for Entry {}

#[async_trait]
impl Store for MemStore {
    type Error = Infallible;
    type Serializer = MemSerializer;

    async fn connect(_uri: &str) -> Result<Self, Self::Error> {
        Ok(Self::default())
    }

    async fn create<T, D>(&self, _descriptor: D) -> Result<(), Self::Error>
    where
        T: StoreData<Self>,
        D: DataDescriptor<T, Self> + Send + Sync,
    {
        Ok(())
    }

    async fn delete<T, Q>(&self, query: Q) -> Result<(), Self::Error>
    where
        T: StoreData<Self> + Send + Sync + 'static,
        Q: DataQuery<T, Self> + Send,
    {
        let mut inner = self.inner.write();
        match inner.get_mut(&T::resource_name()) {
            Some(entries) => {
                let query = serialize_query(query);

                entries.retain(|entry| {
                    for (key, val) in &query.keys {
                        match entry.eq(key, *val) {
                            Some(eq) => {
                                // If the entry doesn't match a single field, keep it.
                                if !eq {
                                    return true;
                                }
                            }
                            // The query key is invalid, keep the entry.
                            None => return true,
                        }
                    }

                    // All checks were `true`, remove the entry.
                    false
                });
            }
            // No entries exist for `T`.
            None => (),
        }

        Ok(())
    }

    async fn get<T, D, Q>(&self, _descriptor: D, query: Q) -> Result<Vec<T>, Self::Error>
    where
        T: StoreData<Self> + Send + Sync + 'static,
        D: DataDescriptor<T, Self> + Send + Sync,
        Q: DataQuery<T, Self> + Send,
    {
        let inner = self.inner.read();
        match inner.get(&T::resource_name()) {
            Some(entries) => {
                let query = serialize_query(query);

                let mut values = Vec::with_capacity(entries.len());

                'outer: for entry in entries {
                    for (key, val) in &query.keys {
                        match entry.keys.get(key) {
                            Some(_) => match entry.eq(key, *val) {
                                Some(eq) => {
                                    if !eq {
                                        // One item doesn't match with the query, try
                                        // the next element.
                                        continue 'outer;
                                    }
                                }
                                // The key in the query doesn't exist on the entry.
                                None => return Ok(Vec::new()),
                            },
                            None => return Ok(Vec::new()),
                        }
                    }

                    // The entry satisfies all requirements of `query`.
                    unsafe {
                        values.push(entry.copy_into());
                    }
                }

                // Shrink values down as much as possible.
                values.shrink_to_fit();
                Ok(values)
            }
            None => Ok(Vec::new()),
        }
    }

    async fn get_all<T, D>(&self, _descriptor: D) -> Result<Vec<T>, Self::Error>
    where
        T: StoreData<Self> + Send + Sync + 'static,
        D: DataDescriptor<T, Self> + Send + Sync,
    {
        let inner = self.inner.read();
        match inner.get(&T::resource_name()) {
            Some(entries) => {
                let mut values = Vec::with_capacity(entries.len());

                for entry in entries {
                    let mut deserializer = MemDeserializer::new_from_slice(&entry.buf);
                    let value = T::deserialize(&mut deserializer)?;

                    values.push(value);
                }

                Ok(values)
            }
            None => Ok(Vec::new()),
        }
    }

    async fn get_one<T, D, Q>(&self, _descriptor: D, query: Q) -> Result<Option<T>, Self::Error>
    where
        T: StoreData<Self> + Send + Sync + 'static,
        D: DataDescriptor<T, Self> + Send,
        Q: DataQuery<T, Self> + Send,
    {
        let inner = self.inner.read();
        match inner.get(&T::resource_name()) {
            Some(entries) => {
                let query = serialize_query(query);

                'outer: for entry in entries {
                    for (key, val) in &query.keys {
                        match entry.keys.get(key) {
                            Some(_) => match entry.eq(key, *val) {
                                Some(eq) => {
                                    if !eq {
                                        // One item doesn't match with the query, try
                                        // the next element.
                                        continue 'outer;
                                    }
                                }
                                // The key in the query doesn't exist on the entry.
                                None => return Ok(None),
                            },
                            None => return Ok(None),
                        }
                    }

                    // The entry satisfies all requirements of `query`.
                    unsafe {
                        return Ok(Some(entry.copy_into()));
                    }
                }

                Ok(None)
            }
            None => Ok(None),
        }
    }

    async fn insert<T>(&self, data: T) -> Result<(), Self::Error>
    where
        T: StoreData<Self> + Send + Sync + 'static,
    {
        let mut serializer = SizeSerializer::new();
        data.serialize(&mut serializer)?;

        let mut serializer = MemSerializer::new(serializer.size);
        data.serialize(&mut serializer)?;

        let entry = Entry {
            buf: serializer.buf,
            keys: serializer.keys,
        };

        let mut inner = self.inner.write();
        match inner.get_mut(&T::resource_name()) {
            Some(vec) => {
                vec.push(entry);
            }
            None => {
                inner.insert(T::resource_name(), vec![entry]);
            }
        }

        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct MemSerializer {
    buf: Vec<u8>,
    keys: HashMap<String, *const u8>,
}

impl MemSerializer {
    fn new(capacity: usize) -> Self {
        Self {
            buf: Vec::with_capacity(capacity),
            keys: HashMap::new(),
        }
    }

    /// Returns the next pointer the buffer.
    unsafe fn next_ptr(&self) -> *const u8 {
        self.buf.as_ptr().add(self.buf.len())
    }

    /// Writes all elements from `iter` into `self.buf` while expecting the buffer
    /// to already have at least the same allocated bytes than the iterator has items.
    ///
    /// # Safety
    /// Calling this method with any [`Iterator`] that has elements causes unidentified
    /// behavoir when `self.buf.capacity() - self.buf.len()` is less than the number of
    /// elements.
    unsafe fn write<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = u8>,
    {
        // Get a `*mut u8` to the first empty location in the Vec.
        let mut ptr = self.buf.as_mut_ptr().add(self.buf.len());

        // Write all elements from iter into the pointer.
        let mut size = 0;
        for item in iter {
            size += 1;

            ptr::write(ptr, item);
            ptr = ptr.add(1);
        }

        // Increment the Vec len by the number of iter elements.
        self.buf.set_len(self.buf.len() + size);
    }
}

impl Serializer<MemStore> for MemSerializer {
    type Error = Infallible;

    fn serialize_bool(&mut self, v: bool) -> Result<(), Self::Error> {
        unsafe {
            self.write([v as u8]);
        }

        Ok(())
    }

    fn serialize_i8(&mut self, v: i8) -> Result<(), Self::Error> {
        unsafe {
            self.write([v as u8]);
        }

        Ok(())
    }

    fn serialize_i16(&mut self, v: i16) -> Result<(), Self::Error> {
        unsafe {
            self.write(v.to_ne_bytes());
        }

        Ok(())
    }

    fn serialize_i32(&mut self, v: i32) -> Result<(), Self::Error> {
        unsafe {
            self.write(v.to_ne_bytes());
        }

        Ok(())
    }

    fn serialize_i64(&mut self, v: i64) -> Result<(), Self::Error> {
        unsafe {
            self.write(v.to_ne_bytes());
        }

        Ok(())
    }

    fn serialize_u8(&mut self, v: u8) -> Result<(), Self::Error> {
        unsafe {
            self.write([v]);
        }

        Ok(())
    }

    fn serialize_u16(&mut self, v: u16) -> Result<(), Self::Error> {
        unsafe {
            self.write(v.to_ne_bytes());
        }

        Ok(())
    }

    fn serialize_u32(&mut self, v: u32) -> Result<(), Self::Error> {
        unsafe {
            self.write(v.to_ne_bytes());
        }

        Ok(())
    }

    fn serialize_u64(&mut self, v: u64) -> Result<(), Self::Error> {
        unsafe {
            self.write(v.to_ne_bytes());
        }

        Ok(())
    }

    fn serialize_f32(&mut self, v: f32) -> Result<(), Self::Error> {
        unsafe {
            self.write(v.to_ne_bytes());
        }

        Ok(())
    }

    fn serialize_f64(&mut self, v: f64) -> Result<(), Self::Error> {
        unsafe {
            self.write(v.to_ne_bytes());
        }

        Ok(())
    }

    fn serialize_str(&mut self, v: &str) -> Result<(), Self::Error> {
        unsafe {
            self.write(v.len().to_ne_bytes());
            // FIXME: It shouldn't be necessary to clone all the bytes.
            self.write(v.as_bytes().to_owned());
        }

        Ok(())
    }

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize<MemStore>,
    {
        let ptr = unsafe { self.next_ptr() };

        self.keys.insert(key.to_string(), ptr);

        value.serialize(self)
    }
}

/// Note: `MemDeserializer` never changes the given buffer. It only copies it.
struct MemDeserializer {
    ptr: *const u8,
}

impl MemDeserializer {
    fn new(ptr: *const u8) -> Self {
        Self { ptr }
    }

    fn new_from_slice(buf: &[u8]) -> Self {
        Self { ptr: buf.as_ptr() }
    }
}

impl MemDeserializer {
    /// Reads `T` from `self.buf` by taking the first from the buffer and transmuting them
    /// into `T`.
    ///
    /// # Safety
    /// It is unidentified behavoir if `self.buf` is smaller than `size_of::<T>()`.
    #[inline]
    unsafe fn read<T>(&mut self) -> T {
        let size = mem::size_of::<T>();

        let value = mem::transmute_copy(&*self.ptr);

        self.ptr = self.ptr.add(size);

        value
    }

    /// Reads the exact number of bytes from the buffer to fill `buf`.
    ///
    /// # Safety
    /// It is unidentified behavoir if `self.ptr` was created from a buffer with a smaller
    /// size.
    #[inline]
    unsafe fn read_exact(&mut self, buf: &mut [u8]) {
        let value = ptr::copy_nonoverlapping(self.ptr, buf.as_mut_ptr(), buf.len());
        self.ptr = self.ptr.add(buf.len());
        value
    }
}

impl Deserializer<MemStore> for MemDeserializer {
    type Error = Infallible;

    fn deserialize_bool(&mut self) -> Result<bool, Self::Error> {
        unsafe { Ok(self.read()) }
    }

    fn deserialize_i8(&mut self) -> Result<i8, Self::Error> {
        unsafe { Ok(self.read()) }
    }

    fn deserialize_i16(&mut self) -> Result<i16, Self::Error> {
        unsafe { Ok(self.read()) }
    }

    fn deserialize_i32(&mut self) -> Result<i32, Self::Error> {
        unsafe { Ok(self.read()) }
    }

    fn deserialize_i64(&mut self) -> Result<i64, Self::Error> {
        unsafe { Ok(self.read()) }
    }

    fn deserialize_u8(&mut self) -> Result<u8, Self::Error> {
        unsafe { Ok(self.read()) }
    }

    fn deserialize_u16(&mut self) -> Result<u16, Self::Error> {
        unsafe { Ok(self.read()) }
    }

    fn deserialize_u32(&mut self) -> Result<u32, Self::Error> {
        unsafe { Ok(self.read()) }
    }

    fn deserialize_u64(&mut self) -> Result<u64, Self::Error> {
        unsafe { Ok(self.read()) }
    }

    fn deserialize_f32(&mut self) -> Result<f32, Self::Error> {
        unsafe { Ok(self.read()) }
    }

    fn deserialize_f64(&mut self) -> Result<f64, Self::Error> {
        unsafe { Ok(self.read()) }
    }

    fn deserialize_string(&mut self) -> Result<String, Self::Error> {
        unsafe {
            let len: usize = self.read();
            let mut bytes = vec![0; len];
            self.read_exact(&mut bytes);

            Ok(String::from_utf8_unchecked(bytes))
        }
    }

    fn deserialize_field<T>(&mut self, _key: &'static str) -> Result<T, Self::Error>
    where
        T: Sized + Deserialize<MemStore>,
    {
        T::deserialize(self)
    }
}

/// A Serializer to predict the exact number of bytes a [`StoreData`] type will have
/// in [`MemStore`].
#[derive(Copy, Clone, Debug, Default)]
struct SizeSerializer {
    size: usize,
}

impl SizeSerializer {
    /// Creates a new `SizeSerializer` with a starting size of `0`.
    fn new() -> Self {
        Self { size: 0 }
    }
}

impl Serializer<MemStore> for SizeSerializer {
    type Error = Infallible;

    fn serialize_bool(&mut self, _: bool) -> Result<(), Self::Error> {
        self.size += mem::size_of::<bool>();
        Ok(())
    }

    fn serialize_i8(&mut self, _: i8) -> Result<(), Self::Error> {
        self.size += mem::size_of::<i8>();
        Ok(())
    }

    fn serialize_i16(&mut self, _: i16) -> Result<(), Self::Error> {
        self.size += mem::size_of::<i16>();
        Ok(())
    }

    fn serialize_i32(&mut self, _: i32) -> Result<(), Self::Error> {
        self.size += mem::size_of::<i32>();
        Ok(())
    }

    fn serialize_i64(&mut self, _: i64) -> Result<(), Self::Error> {
        self.size += mem::size_of::<i64>();
        Ok(())
    }

    fn serialize_u8(&mut self, _: u8) -> Result<(), Self::Error> {
        self.size += mem::size_of::<u8>();
        Ok(())
    }

    fn serialize_u16(&mut self, _: u16) -> Result<(), Self::Error> {
        self.size += mem::size_of::<u16>();
        Ok(())
    }

    fn serialize_u32(&mut self, _: u32) -> Result<(), Self::Error> {
        self.size += mem::size_of::<u32>();
        Ok(())
    }

    fn serialize_u64(&mut self, _: u64) -> Result<(), Self::Error> {
        self.size += mem::size_of::<u64>();
        Ok(())
    }

    fn serialize_f32(&mut self, _: f32) -> Result<(), Self::Error> {
        self.size += mem::size_of::<f32>();
        Ok(())
    }

    fn serialize_f64(&mut self, _: f64) -> Result<(), Self::Error> {
        self.size += mem::size_of::<f64>();
        Ok(())
    }

    fn serialize_str(&mut self, v: &str) -> Result<(), Self::Error> {
        self.size += mem::size_of::<usize>();
        self.size += v.len();
        Ok(())
    }

    fn serialize_field<T>(&mut self, _key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize<MemStore>,
    {
        value.serialize(self)
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum StoreType {
    Bool,
    I8,
    I16,
    I32,
    I64,
    U8,
    U16,
    U32,
    U64,
    F32,
    F64,
    String,
}

struct QuerySerializer {
    buf: Vec<u8>,
    keys: HashMap<String, TypePtr>,
    last_type: StoreType,
}

impl QuerySerializer {
    fn new(capacity: usize) -> Self {
        Self {
            buf: Vec::with_capacity(capacity),
            keys: HashMap::new(),
            // `last_type` should never be used before calling a `serialize_*` method.
            // Therefore the value of this type does not matter.
            last_type: StoreType::Bool,
        }
    }

    /// Returns the pointer to the next allocated but unused element in the buffer.
    #[inline]
    unsafe fn next_ptr(&self) -> *const u8 {
        self.buf.as_ptr().add(self.buf.len())
    }

    /// Writes all elements from `iter` into `self.buf` while expecting the buffer
    /// to already have at least the same allocated bytes than the iterator has items.
    ///
    /// # Safety
    /// Calling this method with any [`Iterator`] that has elements causes unidentified
    /// behavoir when `self.buf.capacity() - self.buf.len()` is less than the number of
    /// elements.
    unsafe fn write<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = u8>,
    {
        // Get a `*mut u8` to the first empty location in the Vec.
        let mut ptr = self.buf.as_mut_ptr().add(self.buf.len());

        // Write all elements from iter into the pointer.
        let mut size = 0;
        for item in iter {
            size += 1;

            ptr::write(ptr, item);
            ptr = ptr.add(1);
        }

        // Increment the Vec len by the number of iter elements.
        self.buf.set_len(self.buf.len() + size);
    }
}

impl Serializer<MemStore> for QuerySerializer {
    type Error = Infallible;

    fn serialize_bool(&mut self, v: bool) -> Result<(), Self::Error> {
        unsafe {
            self.write([v as u8]);
        }

        self.last_type = StoreType::Bool;

        Ok(())
    }

    fn serialize_i8(&mut self, v: i8) -> Result<(), Self::Error> {
        unsafe {
            self.write([v as u8]);
        }

        self.last_type = StoreType::I8;

        Ok(())
    }

    fn serialize_i16(&mut self, v: i16) -> Result<(), Self::Error> {
        unsafe {
            self.write(v.to_ne_bytes());
        }

        self.last_type = StoreType::I16;

        Ok(())
    }

    fn serialize_i32(&mut self, v: i32) -> Result<(), Self::Error> {
        unsafe {
            self.write(v.to_ne_bytes());
        }

        self.last_type = StoreType::I32;
        Ok(())
    }

    fn serialize_i64(&mut self, v: i64) -> Result<(), Self::Error> {
        unsafe {
            self.write(v.to_ne_bytes());
        }

        self.last_type = StoreType::I64;
        Ok(())
    }

    fn serialize_u8(&mut self, v: u8) -> Result<(), Self::Error> {
        unsafe {
            self.write([v]);
        }

        self.last_type = StoreType::U8;
        Ok(())
    }

    fn serialize_u16(&mut self, v: u16) -> Result<(), Self::Error> {
        unsafe {
            self.write(v.to_ne_bytes());
        }

        self.last_type = StoreType::U16;
        Ok(())
    }

    fn serialize_u32(&mut self, v: u32) -> Result<(), Self::Error> {
        unsafe {
            self.write(v.to_ne_bytes());
        }

        self.last_type = StoreType::U32;
        Ok(())
    }

    fn serialize_u64(&mut self, v: u64) -> Result<(), Self::Error> {
        unsafe {
            self.write(v.to_ne_bytes());
        }

        self.last_type = StoreType::U64;
        Ok(())
    }

    fn serialize_f32(&mut self, v: f32) -> Result<(), Self::Error> {
        unsafe {
            self.write(v.to_ne_bytes());
        }

        self.last_type = StoreType::F32;
        Ok(())
    }

    fn serialize_f64(&mut self, v: f64) -> Result<(), Self::Error> {
        unsafe {
            self.write(v.to_ne_bytes());
        }

        self.last_type = StoreType::F64;
        Ok(())
    }

    fn serialize_str(&mut self, v: &str) -> Result<(), Self::Error> {
        unsafe {
            self.write(v.len().to_ne_bytes());
            self.write(v.as_bytes().to_vec());
        }

        self.last_type = StoreType::String;
        Ok(())
    }

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize<MemStore>,
    {
        // Take the next pointer, then serialize the value.
        let ptr = unsafe { self.next_ptr() };

        value.serialize(self)?;

        self.keys
            .insert(key.to_owned(), TypePtr::new(ptr, self.last_type));

        Ok(())
    }
}

fn serialize_query<T, Q>(query: Q) -> QuerySerializer
where
    T: StoreData<MemStore>,
    Q: DataQuery<T, MemStore>,
{
    let mut serializer = SizeSerializer::new();
    query.serialize(&mut serializer).unwrap();

    let mut serializer = QuerySerializer::new(serializer.size);
    query.serialize(&mut serializer).unwrap();

    serializer
}

/// A pointer with type.
#[derive(Copy, Clone, Debug)]
struct TypePtr {
    ptr: *const u8,
    _type: StoreType,
}

impl TypePtr {
    fn new(ptr: *const u8, _type: StoreType) -> Self {
        Self { ptr, _type }
    }
}

// ====================================================
// === Implement [`Serialize`] for supported types. ===
// ====================================================

impl Serialize<MemStore> for bool {
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
    where
        S: Serializer<MemStore>,
    {
        serializer.serialize_bool(*self)
    }

    fn serialize_type<S>(serializer: &mut S) -> Result<(), S::Error>
    where
        S: TypeSerializer<MemStore>,
    {
        serializer.serialize_bool()
    }
}

impl Serialize<MemStore> for i8 {
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
    where
        S: Serializer<MemStore>,
    {
        serializer.serialize_i8(*self)
    }

    fn serialize_type<S>(serializer: &mut S) -> Result<(), S::Error>
    where
        S: TypeSerializer<MemStore>,
    {
        serializer.serialize_i8()
    }
}

impl Serialize<MemStore> for i16 {
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
    where
        S: Serializer<MemStore>,
    {
        serializer.serialize_i16(*self)
    }

    fn serialize_type<S>(serializer: &mut S) -> Result<(), S::Error>
    where
        S: TypeSerializer<MemStore>,
    {
        serializer.serialize_i16()
    }
}

impl Serialize<MemStore> for i32 {
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
    where
        S: Serializer<MemStore>,
    {
        serializer.serialize_i32(*self)
    }

    fn serialize_type<S>(serializer: &mut S) -> Result<(), S::Error>
    where
        S: TypeSerializer<MemStore>,
    {
        serializer.serialize_i32()
    }
}

impl Serialize<MemStore> for i64 {
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
    where
        S: Serializer<MemStore>,
    {
        serializer.serialize_i64(*self)
    }

    fn serialize_type<S>(serializer: &mut S) -> Result<(), S::Error>
    where
        S: TypeSerializer<MemStore>,
    {
        serializer.serialize_i64()
    }
}

impl Serialize<MemStore> for u8 {
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
    where
        S: Serializer<MemStore>,
    {
        serializer.serialize_u8(*self)
    }

    fn serialize_type<S>(serializer: &mut S) -> Result<(), S::Error>
    where
        S: TypeSerializer<MemStore>,
    {
        serializer.serialize_u8()
    }
}

impl Serialize<MemStore> for u16 {
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
    where
        S: Serializer<MemStore>,
    {
        serializer.serialize_u16(*self)
    }

    fn serialize_type<S>(serializer: &mut S) -> Result<(), S::Error>
    where
        S: TypeSerializer<MemStore>,
    {
        serializer.serialize_u16()
    }
}

impl Serialize<MemStore> for u32 {
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
    where
        S: Serializer<MemStore>,
    {
        serializer.serialize_u32(*self)
    }

    fn serialize_type<S>(serializer: &mut S) -> Result<(), S::Error>
    where
        S: TypeSerializer<MemStore>,
    {
        serializer.serialize_u32()
    }
}

impl Serialize<MemStore> for u64 {
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
    where
        S: Serializer<MemStore>,
    {
        serializer.serialize_u64(*self)
    }

    fn serialize_type<S>(serializer: &mut S) -> Result<(), S::Error>
    where
        S: TypeSerializer<MemStore>,
    {
        serializer.serialize_u64()
    }
}

impl Serialize<MemStore> for f32 {
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
    where
        S: Serializer<MemStore>,
    {
        serializer.serialize_f32(*self)
    }

    fn serialize_type<S>(serializer: &mut S) -> Result<(), S::Error>
    where
        S: TypeSerializer<MemStore>,
    {
        serializer.serialize_f32()
    }
}

impl Serialize<MemStore> for f64 {
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
    where
        S: Serializer<MemStore>,
    {
        serializer.serialize_f64(*self)
    }

    fn serialize_type<S>(serializer: &mut S) -> Result<(), S::Error>
    where
        S: TypeSerializer<MemStore>,
    {
        serializer.serialize_f64()
    }
}

impl Serialize<MemStore> for str {
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
    where
        S: Serializer<MemStore>,
    {
        serializer.serialize_str(self)
    }

    fn serialize_type<S>(serializer: &mut S) -> Result<(), S::Error>
    where
        S: TypeSerializer<MemStore>,
    {
        serializer.serialize_str()
    }
}

impl Serialize<MemStore> for String {
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
    where
        S: Serializer<MemStore>,
    {
        serializer.serialize_str(self)
    }

    fn serialize_type<S>(serializer: &mut S) -> Result<(), S::Error>
    where
        S: TypeSerializer<MemStore>,
    {
        serializer.serialize_str()
    }
}

// ======================================================
// === Implement [`Deserialize`] for supported types. ===
// ======================================================

impl Deserialize<MemStore> for bool {
    fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Error>
    where
        D: Deserializer<MemStore>,
    {
        deserializer.deserialize_bool()
    }
}

impl Deserialize<MemStore> for i8 {
    fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Error>
    where
        D: Deserializer<MemStore>,
    {
        deserializer.deserialize_i8()
    }
}

impl Deserialize<MemStore> for i16 {
    fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Error>
    where
        D: Deserializer<MemStore>,
    {
        deserializer.deserialize_i16()
    }
}

impl Deserialize<MemStore> for i32 {
    fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Error>
    where
        D: Deserializer<MemStore>,
    {
        deserializer.deserialize_i32()
    }
}

impl Deserialize<MemStore> for i64 {
    fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Error>
    where
        D: Deserializer<MemStore>,
    {
        deserializer.deserialize_i64()
    }
}

impl Deserialize<MemStore> for u8 {
    fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Error>
    where
        D: Deserializer<MemStore>,
    {
        deserializer.deserialize_u8()
    }
}

impl Deserialize<MemStore> for u16 {
    fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Error>
    where
        D: Deserializer<MemStore>,
    {
        deserializer.deserialize_u16()
    }
}

impl Deserialize<MemStore> for u32 {
    fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Error>
    where
        D: Deserializer<MemStore>,
    {
        deserializer.deserialize_u32()
    }
}

impl Deserialize<MemStore> for u64 {
    fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Error>
    where
        D: Deserializer<MemStore>,
    {
        deserializer.deserialize_u64()
    }
}

impl Deserialize<MemStore> for f32 {
    fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Error>
    where
        D: Deserializer<MemStore>,
    {
        deserializer.deserialize_f32()
    }
}

impl Deserialize<MemStore> for f64 {
    fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Error>
    where
        D: Deserializer<MemStore>,
    {
        deserializer.deserialize_f64()
    }
}

impl Deserialize<MemStore> for String {
    fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Error>
    where
        D: Deserializer<MemStore>,
    {
        deserializer.deserialize_string()
    }
}

#[cfg(test)]
mod tests {

    use super::{MemDeserializer, MemSerializer, MemStore};
    use robbot::store::{delete, get, insert, Deserializer, Serializer, Store};
    use robbot::StoreData;

    use std::mem;

    #[tokio::test]
    async fn test_store() {
        #[derive(Clone, Debug, StoreData, PartialEq, Eq)]
        struct Test {
            a: u8,
            b: i16,
            c: String,
        }

        let store = MemStore::connect("").await.unwrap();

        // The new store should be empty.
        let entries = get!(store, Test).await.unwrap();
        assert_eq!(entries, vec![]);

        // Insert a single item.
        let data = Test {
            a: 23,
            b: -545,
            c: String::from("abcd"),
        };
        insert!(store, data.clone()).await.unwrap();

        let entries = get!(store, Test).await.unwrap();
        assert_eq!(entries, vec![data.clone()]);

        // Insert a second item.
        let data2 = Test {
            a: 74,
            b: 2343,
            c: String::from("Hello World!"),
        };
        insert!(store, data2.clone()).await.unwrap();

        let entries = get!(store, Test).await.unwrap();
        assert_eq!(entries, vec![data.clone(), data2.clone()]);

        // Get the item where a == 74 (data2).
        let entry = get!(store, Test=> {
            a == 74
        })
        .await
        .unwrap();
        assert_eq!(entry, vec![data2.clone()]);

        // Remove the entry where c == "abcd" (data).
        delete!(store, Test => {
            c == "abcd".to_string(),
        })
        .await
        .unwrap();

        let entries = get!(store, Test).await.unwrap();
        assert_eq!(entries, vec![data2]);
    }

    #[test]
    fn test_serializer() {
        let mut serializer = MemSerializer::new(mem::size_of::<(u8, i8, u16)>());
        serializer.serialize_u8(32).unwrap();
        serializer.serialize_i8(-3).unwrap();
        serializer.serialize_u16(256).unwrap();

        #[cfg(target_endian = "little")]
        assert_eq!(serializer.buf, vec![32, 253, 0, 1]);

        #[cfg(target_endian = "big")]
        assert_eq!(serializer.buf, vec![32, 253, 1, 0]);

        let mut serializer = MemSerializer::new(mem::size_of::<usize>() + "Hello World!".len());
        serializer.serialize_str("Hello World!").unwrap();

        #[cfg(target_endian = "little")]
        assert_eq!(
            serializer.buf,
            vec![
                12, 0, 0, 0, 0, 0, 0, 0, b'H', b'e', b'l', b'l', b'o', b' ', b'W', b'o', b'r',
                b'l', b'd', b'!'
            ]
        );

        #[cfg(target_endian = "big")]
        assert_eq!(
            serializer.buf,
            vec![
                0, 0, 0, 0, 0, 0, 0, 12, b'H', b'e', b'l', b'l', b'o', b' ', b'W', b'o', b'r',
                b'l', b'd', b'!'
            ]
        );
    }

    #[test]
    fn test_deserializer() {
        #[cfg(target_endian = "little")]
        let mut deserializer = MemDeserializer::new_from_slice(&[32, 253, 0, 1]);

        #[cfg(target_endian = "big")]
        let mut deserializer = MemDeserializer::new(&[32, 253, 1, 0]);

        assert_eq!(deserializer.deserialize_u8().unwrap(), 32);
        assert_eq!(deserializer.deserialize_i8().unwrap(), -3);
        assert_eq!(deserializer.deserialize_u16().unwrap(), 256);

        #[cfg(target_endian = "little")]
        let input = &[
            12, 0, 0, 0, 0, 0, 0, 0, b'H', b'e', b'l', b'l', b'o', b' ', b'W', b'o', b'r', b'l',
            b'd', b'!',
        ];

        #[cfg(target_endian = "big")]
        let input = &[
            0, 0, 0, 0, 0, 0, 0, 12, b'H', b'e', b'l', b'l', b'o', b' ', b'W', b'o', b'r', b'l',
            b'd', b'!',
        ];

        let mut deserializer = MemDeserializer::new_from_slice(input);
        assert_eq!(deserializer.deserialize_string().unwrap(), "Hello World!");
    }
}
