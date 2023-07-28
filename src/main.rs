use extendable_hashmap::HashMap;

fn main() {
    let mut map: HashMap<i32, i32> = HashMap::new();

    let mut map = HashMap::new();
    for i in 0..30 {
        assert!(map.remove(&i).is_none());
        map.insert(i, i);
    }
    assert_eq!(map.len(), 30);

    for i in 0..30 {
        assert_eq!(map.get(&i), Some(&i));
    }

    println!("{:?}", map);

    for i in 0..30 {
        // bug
        assert_eq!(map.remove(&i), Some(i));
        println!("删除{i}");
        println!("{:?}", map);
    }

    assert_eq!(map.len(), 0);
}
