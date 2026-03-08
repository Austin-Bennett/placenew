use std::marker::PhantomData;
use std::mem::MaybeUninit;
use placenew::place_boxed;

pub struct TestStruct {
    trivial: i32,
    array: [i32; 100_000],
    explicit_array: [i32; 10],
    nested_array: [[i32; 100_000]; 10],
    complex: Vec<i32>,
    array_complex: [Vec<i32>; 10],
    nested_array_complex: [[Vec<i32>; 100]; 10],
    unit: (),
    phantom: PhantomData<u8>,
    boxed: Box<[i32; 100_000]>,
    nested_box: Box<Box<i32>>,
}


fn main() {

    let my_boxed_slice = place_boxed!([10; 100_000], [i32; 100_000]);

    let b = place_boxed!(

        TestStruct{
            trivial: 10,
            array: [5; 100_000],
            explicit_array: [1, 2, 3, 4, 5, 6, 7, 8, 9, 10],
            nested_array: [[10; 100_000]; 10],
            complex: Vec::new(),
            array_complex: std::array::from_fn(|_| Vec::new()),
            nested_array_complex: std::array::from_fn(|_| std::array::from_fn(|_| Vec::new())),
            unit: (),
            phantom: PhantomData,
            boxed: place_boxed!([10; 100_000], [i32; 100_000]),
            nested_box: place_boxed!( place_boxed!(10, i32), Box<i32>),
        }
    );

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


    assert_eq!(b.unit, ());
    assert_eq!(b.phantom, PhantomData);
    for i in 0..100_000 {
        assert_eq!(b.boxed[i], 10);
    }

    assert_eq!(*(*(b.nested_box)), 10);
    println!("All tests passed!")
}
