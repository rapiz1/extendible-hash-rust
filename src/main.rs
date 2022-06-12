use extendible_hash::HashTable;

fn main() {
    let mut ht = HashTable::<i32, i32>::new();

    let range = -(1 << 8)..(1 << 8);

    for i in range.clone() {
        ht.put(i, i * 2);
        println!("[{}] = {:?} inserted", i, ht.get(&i));
    }

    for i in range.clone() {
        ht.delete(&i);
        println!("[{}] = {:?} deleted", i, ht.get(&i));
    }

    println!("Hello, world!");
}
