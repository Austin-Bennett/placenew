use placenew::place_boxed;

pub struct TestStruct {
    trivial: i32,
    array: [i32; 100_000],
    explicit_array: [i32; 10],
    nested_array: [[i32; 100_000]; 100_000],
    complex: Vec<i32>,
    array_complex: [Vec<i32>; 10],
    nested_array_complex: [[Vec<i32>; 100]; 10],
}

struct MyStruct {
    trivial_val: i32,
    name: String,
    array: [i32; 5],
    nested_array: [[i32; 10]; 5]
}

fn test_my_struct() {
    let my_box = unsafe{ place_boxed!(
        MyStruct {
            trivial_val: 0,
            name: String::from("Bob"),
            array: [1, 2, 3, 4, 5],
            nested_array: [[5; 10]; 5]
        }
    ) };
}

fn main() {
    let b = unsafe{ place_boxed!(TestStruct{
        trivial: 10,
        array: [5; 100_000],
        explicit_array: [1, 2, 3, 4, 5, 6, 7, 8, 9, 10],
        nested_array: [[10; 100_000]; 100_000],
        complex: Vec::new(),
        array_complex: [Vec::new(); 10],
        nested_array_complex: [[Vec::new(); 100]; 10]
    }) };
}
