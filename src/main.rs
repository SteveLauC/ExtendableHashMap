use extendable_hashmap::HashMap;

fn main() {
    let mut map: HashMap<i32, i32> = HashMap::new();

    for i in 0..10000 {
        map.insert(i, i);
    }
}
