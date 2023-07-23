use extendable_hashmap::HashMap;

fn main() {
    let mut map: HashMap<i32, i32> = HashMap::new();
    map.insert(1, 1);
    map.insert(2, 2);

    println!("{:?}", map);
}
