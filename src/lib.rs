extern crate proc_macro;
use proc_macro::{TokenStream};
use quote::{format_ident, quote};
use syn::{parse_macro_input, Expr, FieldValue, Member};

/*
This macro allows you to construct values in the heap in-place safely,
using the struct initialization syntax


Example usage:
let mut my_box = place_boxed!(
    MyStruct{
        member_1: 5,
        member_array: [0; 100],
        member_array2: [1, 2, 3],
    }
)

Example codegen:
let mut my_box = {
    let mut res = Box::<MyStruct>::new_uninit();

    unsafe {
        let ptr = res.as_mut_ptr();

        (&raw mut (*ptr).member_1).write(5);

        for i in 0..100 {
            (&raw mut (*ptr).member_array[i]).write(0);
        }

        (&raw mut (*ptr).member_array2[0]).write(1);
        (&raw mut (*ptr).member_array2[1]).write(2);
        (&raw mut (*ptr).member_array2[2]).write(3);

        res.assume_init()
    }
}
*/



fn inner_place_expr(member: proc_macro2::TokenStream, val: &Expr, nesting: u32) -> proc_macro2::TokenStream {
    match val {
        Expr::Array(arr) => {
            let construction = arr
                .elems
                .iter()
                .enumerate()
                .map(|(i, item)| {
                    inner_place_expr(quote! { #member[#i] }, item, nesting + 1)
                });

            quote! {
                #(#construction)*
            }
        },
        Expr::Repeat(rep) => {
            let v = &rep.expr;
            let len = &rep.len;
            let loop_var = format_ident!("i_{}", nesting);
            let initializer = inner_place_expr(quote! { #member[#loop_var] }, &v, nesting + 1);
            quote! {
                for #loop_var in 0..#len {
                    #initializer
                }
            }
        },
        expr => {
            quote! {
                (&raw mut #member).write(#expr);
            }
        }
    }
}


/// place_boxed is a macro that creates a structure in-place from a structure initializer input
///
/// this macro is unsafe as there isn't a way to validate the structure initializer input is valid,
/// you can leave out some fields, and so this macro requires wrapping with unsafe in order to
/// mark that the programmer must validate their input is valid, my recommendation is to
/// first initialize the struct as you would normally, validate it compiles through either your ide,
/// or manually compiling, then wrapping in place_boxed
///
/// this macro is great for reducing unnecessary copying, or simplifying constructing
/// large structures on the heap
///
/// example usage:
/// ```rust
///struct MyStruct {
///    trivial_val: i32,
///    name: String,
///    array: [i32; 5],
///    nested_array: [[i32; 10]; 5]
///}
///
///let my_box = unsafe{ place_boxed!(
///    MyStruct {
///        trivial_val: 10,
///        name: String::from("Bob"),
///        array: [1, 2, 3, 4, 5],
///        nested_array: [[5; 10]; 5]
///    }
///) };
/// ```
///
///example codegen (edited for readability):
/// ```rust
///let my_box = unsafe{ {
///    let mut res = std::boxed::Box::<MyStruct>::new_uninit();
///
///    let ptr = res.as_mut_ptr();
///
///    (&raw mut (*ptr).trivial_val).write(10);
///
///    (&raw mut (*ptr).name).write(String::from("Bob"));
///
///    (&raw mut (*ptr).array[0usize]).write(1);
///
///    (&raw mut (*ptr).array[1usize]).write(2);
///
///    (&raw mut (*ptr).array[2usize]).write(3);
///
///    (&raw mut (*ptr).array[3usize]).write(4);
///
///    (&raw mut (*ptr).array[4usize]).write(5);
///
///    for i_0 in 0..5 {
///        for i_1 in 0..10 {
///            (&raw mut (*ptr).nested_array[i_0][i_1]).write(5);
///        }
///    }
///
///    res.assume_init()
///} }
/// ```
#[proc_macro]
pub fn place_boxed(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as Expr);

    let struct_expr = match input {
        syn::Expr::Struct(s) => s,
        _ => return syn::Error::new_spanned(input, "Expected structure initializer")
            .to_compile_error()
            .into()
    };

    if let Some(rest) = &struct_expr.rest {
        return syn::Error::new_spanned(rest, "place_boxed limitation: Cannot use \
        ..Rest expressions in struct initializers")
            .to_compile_error()
            .into();
    }



    let path = &struct_expr.path;


    let mut generated = Vec::new();

    // std::boxed::Box::<i32>::new_uninit()



    for field in &struct_expr.fields {
        let name = &field.member;
        generated.push(
            inner_place_expr(
                quote!{
                    (*ptr).#name
                },
                &field.expr, 0
            )
        )
    }




    quote! {
        {
            let mut res = std::boxed::Box::<#path>::new_uninit();

            let ptr = res.as_mut_ptr();

            #(#generated)*

            res.assume_init()

        }
    }.into()
}


