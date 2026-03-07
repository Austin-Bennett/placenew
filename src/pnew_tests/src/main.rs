use placenew::place_boxed;

pub struct TestStruct {
    trivial: i32,
    array: [i32; 100_000],
    explicit_array: [i32; 10],
    nested_array: [[i32; 100_000]; 10],
    complex: Vec<i32>,
    array_complex: [Vec<i32>; 10],
    nested_array_complex: [[Vec<i32>; 100]; 10],
}


fn main() {
    let b = unsafe{ place_boxed!(TestStruct{
        trivial: 10,
        array: [5; 100_000],
        explicit_array: [1, 2, 3, 4, 5, 6, 7, 8, 9, 10],
        nested_array: [[10; 100_000]; 10],
        complex: Vec::new(),
        array_complex: [Vec::new(); 10],
        nested_array_complex: [[Vec::new(); 100]; 10]
    }) };

    assert_eq!(b.trivial, 10);
    for i in 0..5 {
        assert_eq!(b.array[i], 5);
    }

    assert_eq!([1, 2, 3, 4, 5, 6, 7, 8, 9, 10], b.explicit_array);
    for i in 0..10 {
        for j in 0..100_000 {
            assert_eq!(b.nested_array[i][j], 10);
        }
    }
    assert_eq!(b.complex.len(), 0);
    for i in 0..10 {
        assert_eq!(b.array_complex[i].len(), 0);
    }
    for i in 0..10 {
        for j in 0..100 {
            assert_eq!(b.nested_array_complex[i][j].len(), 0);
        }
    }
    println!("All tests passed!")
}
