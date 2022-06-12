/*
use std::mem::size_of;

const CACHE_SIZE: usize = 4096;

const fn fit_in_cache<T>() -> usize {
    CACHE_SIZE / size_of::<T>()
}

struct Bucket<T> {
    data: [T; fit_in_cache::<T>()],
}
*/

use std::{
    cell::UnsafeCell,
    collections::hash_map::DefaultHasher,
    fmt::Display,
    hash::{Hash, Hasher},
    rc::Rc,
};

const BUCKET_SIZE: usize = 32;
struct Bucket<K, V> {
    data: [Option<(K, V)>; BUCKET_SIZE],
    size: usize,
    depth: usize,
}

impl<K: Eq + Clone + Hash, V: Clone> Bucket<K, V> {
    fn hash(&self, k: &K) -> u64 {
        let mut s = DefaultHasher::new();
        k.hash(&mut s);
        s.finish() & bitmask(self.depth)
    }

    fn new() -> Bucket<K, V> {
        Bucket {
            data: Default::default(),
            size: 0,
            depth: 0,
        }
    }

    fn get(&self, k: &K) -> Option<V> {
        let r = self.data.iter().find(|x| match x.as_ref() {
            Some((nk, _)) => k == nk,
            None => false,
        });
        unsafe {
            return r.map(|o| o.as_ref().unwrap_unchecked().1.clone());
        }
    }

    /// Returns if successful
    fn put(&mut self, k: K, v: V) -> bool {
        let ret = self.data.iter_mut().find(|o| o.is_none());
        match ret {
            Some(pos) => {
                *pos = Some((k, v));
                self.size += 1;
                true
            }
            None => {
                panic!("the bucket is full")
            }
        }
    }

    fn full(&self) -> bool {
        self.size == BUCKET_SIZE
    }

    /// Split out a new bucket
    fn split(&mut self) -> Bucket<K, V> {
        let mut b = Bucket::<K, V>::new();
        b.depth = self.depth + 1;
        self.depth += 1;

        let mut i = 0;
        while i < BUCKET_SIZE {
            match self.data[i].as_ref() {
                Some((k, v)) => {
                    let hv = self.hash(k);
                    if hv & (1 << (self.depth - 1)) != 0 {
                        drop(k);
                        drop(v);

                        unsafe {
                            let kv = self.data[i].take().unwrap_unchecked();
                            b.put(kv.0, kv.1);
                            self.size -= 1;
                        }
                    }
                }
                None => (),
            }

            i += 1;
        }
        b
    }

    pub fn delete(&mut self, key: &K) {
        let ret = self.data.iter_mut().find(|o| match o {
            Some((k, _)) => k == key,
            None => false,
        });

        match ret {
            Some(pos) => {
                let _ = pos.take();
            }
            None => (),
        }
    }
}

pub struct HashTable<K, V> {
    table: Vec<Rc<UnsafeCell<Bucket<K, V>>>>,
    depth: usize,
}

const fn bitmask(depth: usize) -> u64 {
    (1 << depth) - 1
}

impl<K: Hash + Eq + Clone + Display, V: Clone> HashTable<K, V> {
    fn hash(&self, k: &K) -> u64 {
        let mut s = DefaultHasher::new();
        k.hash(&mut s);
        s.finish() & bitmask(self.depth)
    }

    pub fn new() -> HashTable<K, V> {
        let b = Rc::new(Bucket::<K, V>::new().into());
        let ht = HashTable {
            table: vec![b],
            depth: 0,
        };
        ht
    }

    /// Get the value associate with the key
    pub fn get(&mut self, key: &K) -> Option<V> {
        unsafe { (*self.table[self.hash(key) as usize].get()).get(key) }
    }

    fn reserve(&mut self, k: &K) {
        loop {
            let idx = self.hash(k) as usize;
            let full;
            let local_depth;
            unsafe {
                let b = self.table[idx].get();
                full = (*b).full();
                local_depth = (*b).depth;
            }
            if full {
                //println!("{} {}", self.depth, self.table.len());
                if local_depth == self.depth {
                    //println!("{} cause global grow", idx & !(1 << local_depth));
                    self.table.reserve(self.table.len());
                    for i in 0..self.table.len() {
                        self.table.push(self.table[i as usize].clone());
                    }
                    self.depth += 1;
                }

                if local_depth < self.depth {
                    unsafe {
                        let b0 = Rc::as_ptr(&self.table[idx & bitmask(local_depth) as usize])
                            as *mut Bucket<K, V>;
                        let b1 = Rc::new(UnsafeCell::new((*b0).split()));
                        // println!(
                        //     "splited {:#012b}, local_depth: {}, global_depth: {}",
                        //     idx & bitmask(local_depth) as usize,
                        //     local_depth,
                        //     self.depth
                        // );
                        for prefix in 0..(1 << (self.depth - local_depth - 1)) {
                            let i = (idx & bitmask(local_depth) as usize)
                                | 1 << local_depth
                                | (prefix << (local_depth + 1)) as usize;
                            // println!("update {:#012b}", i);
                            self.table[i] = b1.clone();
                        }
                    }
                }
            } else {
                break;
            }
        }
    }

    /// Put the key value pair into the table
    pub fn put(&mut self, key: K, val: V) {
        self.reserve(&key);
        let hv = self.hash(&key) as usize;
        unsafe {
            (*self.table[hv].get()).put(key, val);
        }
    }

    pub fn delete(&mut self, key: &K) {
        unsafe { (*self.table[self.hash(key) as usize].get()).delete(key) }
    }
}
