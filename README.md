# placenew

**A rust crate providing safe placement-new semantics for rust**
This crate is a procedural macro crate that provides the proc macro: `place_boxed`

The macro takes a struct initializer as an input, and generates code that initalizes the items in-place in heap memory

## Usecase
In some cases, you may want to initialize memory in-place on the heap in order to avoid an expensive copy operation,
such is the case in this code:
```rust
let mut res = Box::new(
    MyStruct::new()
)
```
In some cases the rust compiler will optomize away the copy, but its not guaranteed, and assuming it is can often lead
to undefined behavior, that is what this crate aims to solve

## Examples

**Example 1**
```rust
pub struct BigStruct {
    array: [u64; 200_000], //array of 200_000 u64's
    array2: [i32; 5],
    number: i32,
}

impl BigStruct {

  pub fn new() -> Self {
    place_boxed!{
      Self{
        array: [0; 200_000],
        array2: [1, 2, 3, 4, 5],
        number: 0x91ACE,
      }
    }
  }

}
```

The macro will then generate code that looks like:
```rust
pub struct BigStruct {
    array: [u64; 200_000], //array of 200_000 u64's
    number: i32,
}

impl BigStruct {

  pub fn new() -> Self {
    place_boxed!{
      {
        let mut res = std::boxed::Box::<Self>::new_uninit();
        unsafe {
          let ptr = res.as_mut_ptr();
          //array: [0; 200_000]
          for i in 0..200_000 {
            (&raw mut (*ptr).array[i]).write(0)
          }

          //array2: [1, 2, 3, 4, 5]
          (&raw mut (*ptr).array2[0]).write(1)
          (&raw mut (*ptr).array2[1]).write(2)
          (&raw mut (*ptr).array2[2]).write(3)
          (&raw mut (*ptr).array2[3]).write(4)
          (&raw mut (*ptr).array2[4]).write(5)

          //number: 0x91ACE
          (&raw mut (*ptr).number).write(0x91ACE)

          res.assume_init()
        }
      }
    }
  }

}
```



## Limitations

The syntax:
```rust
place_boxed!(
  MyStruct {
    ..Rest()
  }
)
```

is not allowed, this is because there isnt really a good way to figure out which values are initalized by the Rest statement, and which are not
if a good method can be figure out, I will add it to the crate and remove this limitation
