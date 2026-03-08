# placenew

**A rust crate providing safe placement-new semantics for rust**
This crate is a procedural macro crate that provides the proc macro: `place_boxed`

The macro takes a struct initializer as an input, and generates code that initializes the items in-place in heap memory

## Usecase
In some cases, you may want to initialize memory in-place on the heap in order to avoid an expensive copy operation,
such is the case in this code:
```rust
let mut res = Box::new(
    MyStruct::new()
)
```
In some cases the rust compiler will optimize away the copy, but it's not guaranteed, and assuming it is can often lead
to undefined behavior, that is what this crate aims to solve

## Examples

**Example 1**
```rust
struct MyStruct {
    trivial_val: i32,
    name: String,
    array: [i32; 5],
    nested_array: [[i32; 10]; 5]
}

let my_box = place_boxed!(
    MyStruct {
        trivial_val: 10,
        name: String::from("Bob"),
        array: [1, 2, 3, 4, 5],
        nested_array: [[5; 10]; 5]
    }
);
```

**Example codegen (edited for readability):**
```rust
let my_box = unsafe {

    let _ensure_struct_correct = || {
         MyStruct {
            trivial_val: 10,
            name: String::from("Bob"),
            array: [1, 2, 3, 4, 5],
            nested_array: [[5; 10]; 5]
         }
    };

    let mut res = std::boxed::Box::<MyStruct>::new_uninit();

    let ptr = res.as_mut_ptr();

    (&raw mut (*ptr).trivial_val).write(10);

    (&raw mut (*ptr).name).write(String::from("Bob"));

    (&raw mut (*ptr).array[0usize]).write(1);

    (&raw mut (*ptr).array[1usize]).write(2);

    (&raw mut (*ptr).array[2usize]).write(3);

    (&raw mut (*ptr).array[3usize]).write(4);

    (&raw mut (*ptr).array[4usize]).write(5);

    for i_0 in 0..5 {
        for i_1 in 0..10 {
            (&raw mut (*ptr).nested_array[i_0][i_1]).write(5);
        }
    }

    res.assume_init()
}
```


**As of 2.0.0 you can also create non-structure types with the macro like so**
```rust
let my_boxed_slice = place_boxed!(
    [0; 100_000],  //the initializing value
    [i32, 100_000] //the type of the value
);
```

**Codegen:**
```rust
let my_boxed_slice = unsafe {
    let _ensure_correct = || { [10; 100_000] };

    let mut res = std::boxed::Box::<[i32; 100_000]>::new_uninit();
    let ptr = res.as_mut_ptr();
    for i_0 in 0..100_000 { (&raw mut (*ptr)[i_0]).write(10); }
    res.assume_init()
}
```