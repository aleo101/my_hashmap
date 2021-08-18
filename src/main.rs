use std::convert::TryInto;
use std::mem;

const MAP_FULL: i16 = -2;
const MAP_MISSING: i16 = -3;
const MAP_OMEM: i16 = -1;
const MAP_OK: i16 = 0;
const INIT_CAP: usize = 1024;

type MapT<T> = HashMapMap<T>;

#[repr(C)]
#[derive(Default, Clone)]
struct HashMapElement<T> {
    key: usize,
    in_use: i32,
    data: T,
}

#[repr(C)]
struct HashMapMap<T> {
    table_size: usize,
    size: i32,
    data: Vec<HashMapElement<T>>,
}

fn hashmap_new<T>() -> MapT<T>
where
    T: Clone + Default,
{
    let m = HashMapMap {
        table_size: INIT_CAP,
        size: 0,
        data: vec![HashMapElement::<T>::default(); INIT_CAP],
    };

    m
}

fn hashmap_hash_int<T>(m: &HashMapMap<T>, mut key: usize) -> usize {
    /* Robert senkins' 32 bit Mix Function */
    key += key << 12;
    key ^= key >> 22;
    key += key << 4;
    key ^= key >> 9;
    key += key << 10;
    key ^= key >> 2;
    key += key << 7;
    key ^= key >> 12;

    /* Knuth's Multiplicative Method */
    key = (key >> 3) * 2654435761;

    key % m.table_size
}

fn hashmap_hash<T>(m: &MapT<T>, key: usize) -> i16 {
    if m.size == m.table_size.try_into().unwrap() {
        return MAP_FULL;
    }
    let mut curr: usize = hashmap_hash_int(&m, key);
    for _ in 0..m.table_size {
        if m.data[curr].in_use == 0 {
            return curr as i16;
        }
        if m.data[curr].key == key && m.data[curr].in_use == 1 {
            return curr as i16;
        }

        curr = curr + 1 % m.table_size
    }

    MAP_FULL
}

fn hashmap_rehash<T>(m: &mut MapT<T>) -> i16
where
    T: Default + Copy,
{
    let mut curr = vec![HashMapElement::<T>::default(); 2 * INIT_CAP];
    // let curr point to old data in memory
    //let data field of m now point to new default-init'd vector.
    mem::swap(&mut m.data, &mut curr);
    let old_size = m.table_size;
    m.table_size = 2 * m.table_size;
    m.size = 0;

    for i in 0..old_size {
        let status: i16 = hashmap_put(m, curr[i].key, curr[i].data);
        if status != MAP_OK {
            return status;
        }
    }

    return MAP_OK;
}

fn hashmap_put<T>(m: &mut MapT<T>, key: usize, value: T) -> i16
where
    T: Clone + Default + Copy,
{
    let mut index = hashmap_hash(&m, key);
    while index == MAP_FULL {
        if hashmap_rehash(m) == MAP_OMEM {
            return MAP_OMEM;
        }
        index = hashmap_hash(m, key);
    }
    m.data[index as usize].data = value.clone();
    m.data[index as usize].key = key;
    m.data[index as usize].in_use = 1;
    m.size += 1;
    return MAP_OK;
}

fn hashmap_get<T>(m: &mut MapT<T>, key: usize) -> Option<T>
where
    T: Clone + Default + Copy,
{
    let mut curr = hashmap_hash_int(&m, key);
    for _ in 0..m.table_size {
        if m.data[curr].key == key && m.data[curr].in_use == 1 {
            return Some(m.data[curr].data);
        }
        curr = (curr + 1) % m.table_size;
    }
    None
}

fn hashmap_get_one<T>(m: &mut MapT<T>, remove: usize) -> Option<T>
where
    T: Clone + Default + Copy,
{
    if hashmap_length(m) == 0 {
        return None;
    }

    for i in 0..m.table_size {
        if m.data[i].in_use != 0 {
            if remove != 0 {
                m.data[i].in_use = 0;
                m.size -= 1;
            }
            return Some(m.data[i].data);
        }
    }
    None
}

fn hashmap_remove<T>(m: &mut MapT<T>, key: usize) -> i16
where
    T: Default,
{
    let mut curr = hashmap_hash_int(m, key);
    for _ in 0..m.table_size {
        if m.data[curr].key == key && m.data[curr].in_use == 1 {
            /* Blank out the fields */
            m.data[curr].in_use = 0;
            m.data[curr].data = T::default();
            m.data[curr].key = 0;
            /* Reduce the size */
            m.size -= 1;
            return MAP_OK;
        }
        curr = (curr + 1) % m.table_size;
    }

    MAP_MISSING
}
fn hashmap_length<T>(m: &MapT<T>) -> i32 {
    m.size
}
fn main() {
    let map = &mut hashmap_new::<i32>();
    hashmap_put(map, 1, 4);
    println!(
        "Getting random (first?) element: {}",
        hashmap_get_one(map, 0).unwrap()
    );
    println!(
        "Getting random (first?) element: {}",
        hashmap_get_one(map, 0).unwrap()
    );
    hashmap_put(map, 2, 365);
    println!(
        "Getting random (first?) element: {}",
        hashmap_get_one(map, 0).unwrap()
    );
    println!(
        "Getting element with key 2: {}",
        hashmap_get(map, 2).unwrap()
    );
    hashmap_remove(map, 2);
    println!(
        "Getting random (first?) element: {}",
        hashmap_get_one(map, 0).unwrap()
    );
}
