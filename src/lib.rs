extern crate proc_macro;
use proc_macro::{TokenStream};
use std::convert::TryInto;
use proc_macro2::Span;
use quote::{format_ident, quote};
use syn::{parse_macro_input, Expr, FieldValue, Member, Type, TypePath};
use syn::parse::Parser;
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

struct PlaceKnownInput {
    expr: syn::Expr,
    ty: Option<syn::Type>,
}

struct PlaceIntoInput {
    ptr: syn::Expr,
    expr: syn::Expr,
}

impl syn::parse::Parse for PlaceKnownInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let expr = input.parse::<syn::Expr>()?;
        let ty = if input.peek(syn::Token![,]) {
            input.parse::<syn::Token![,]>()?;
            Some(input.parse::<syn::Type>()?)
        } else {
            None
        };
        Ok(Self { expr, ty })
    }
}

impl syn::parse::Parse for PlaceIntoInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let ptr = input.parse::<syn::Expr>()?;

        let expr = if input.peek(syn::Token![,]) {
            input.parse::<syn::Token![,]>()?;
            input.parse::<syn::Expr>()?
        } else {
            return Err(syn::Error::new(Span::call_site(), "Expected expression to place into"))
        };
        Ok(Self { ptr, expr })
    }
}


/// place_boxed is a macro that creates a structure in-place from a structure initializer input
///
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
///let my_box = = place_boxed!(MyStruct{
///         trivial_val: 100,
///         name: String::from("Jeff"),
///         array: [1, 2, 3, 4, 5],
///         nested_array: [[0; 10]; 5]
///     });
///
/// let my_boxed_slice = place_boxed!( [10; 100_000], [i32; 100_000] );
/// ```
///
///example codegen (edited for readability):
/// ```rust
///let my_box = unsafe {
///         let mut res = std::boxed::Box::<MyStruct>::new_uninit();
///         let ptr = res.as_mut_ptr();
///
///         //inner place_into! expansion
///         {
///             let _ensure_struct_correct = || {
///                 MyStruct {
///                     trivial_val: 100,
///                     name: String::from("Jeff"),
///                     array: [1, 2, 3, 4, 5],
///                     nested_array: [[0; 10]; 5],
///                 }
///             };
///
///
///             let ptr = ptr;
///
///             (&raw mut (*ptr).trivial_val).write(100);
///
///             (&raw mut (*ptr).name).write(String::from("Jeff"));
///
///             (&raw mut (*ptr).array[0usize]).write(1);
///
///             (&raw mut (*ptr).array[1usize]).write(2);
///
///             (&raw mut (*ptr).array[2usize]).write(3);
///
///             (&raw mut (*ptr).array[3usize]).write(4);
///
///             (&raw mut (*ptr).array[4usize]).write(5);
///
///             for i_0 in 0..5 { for i_1 in 0..10 { (&raw mut (*ptr).nested_array[i_0][i_1]).write(0); } }
///         }
///
///         res.assume_init()
///     };
///
/// let my_boxed_slice = unsafe {
///         let mut res = std::boxed::Box::<[i32; 100_000]>::new_uninit();
///         let ptr = res.as_mut_ptr();
///
///         //inner place_into! expansion
///         {
///             let _ensure_correct = || { [10; 100_000] };
///
///             let ptr = ptr;
///
///             for i_0 in 0..100_000 { (&raw mut (*ptr)[i_0]).write(10); }
///         }
///
///         res.assume_init()
///     };
/// ```
///
///  see also: place_into!
#[proc_macro]
pub fn place_boxed(input: TokenStream) -> TokenStream {
    let inp = parse_macro_input!(input as PlaceKnownInput);

    let ty = if let Some(ty) = inp.ty {
        ty
    } else {
        let Expr::Struct(s) = &inp.expr else { return syn::Error::new_spanned(inp.expr, "Expected type argument for constructing non-structure type")
            .to_compile_error().into() };

        Type::Path(TypePath{
            qself: None,
            path: s.path.clone()
        })
    };

    let expr = inp.expr;


    quote! {

        unsafe{
            let mut res = std::boxed::Box::<#ty>::new_uninit();

            let ptr = res.as_mut_ptr();

            place_into!(ptr, #expr);

            res.assume_init()
        }

    }.into()
}


/// place_into! is a macro that generates code to construct a value in-place at a specified pointer location.
/// you must wrap it in an unsafe block, as the macro does not or can not verify that the pointer
/// is both properly aligned and contains enough space for the data to be constructed there/
///
/// example usage:
/// ```rust
/// let my_slice_allocation = unsafe{ alloc(Layout::new::<[i32; 100_000]>()) as *mut [i32; 100_000] };
///
/// unsafe{ place_into!(my_slice_allocation, [10; 100_000]); }
///
/// let my_alloced_slice = unsafe{ &*my_slice_allocation };
///
/// for i in 0..100_000 {
///     assert_eq!(my_alloced_slice[i], 10)
/// }
/// ```
///
/// example codegen (edited for readability):
/// ```rust
/// let my_slice_allocation = unsafe{ alloc(Layout::new::<[i32; 100_000]>()) as *mut [i32; 100_000] };
///
/// unsafe{
///     {
///         let _ensure_correct = || { [10; 100_000] };
///         let ptr = my_slice_allocation;
///         for i_0 in 0..100_000 { (&raw mut (*ptr)[i_0]).write(10); }
///     }
/// }
///
/// let my_alloced_slice = unsafe{ &*my_slice_allocation };
///
/// for i in 0..100_000 {
///     assert_eq!(my_alloced_slice[i], 10)
/// }
/// ```
///
/// see also: place_boxed!
#[proc_macro]
pub fn place_into(input: TokenStream) -> TokenStream {

    let inp = parse_macro_input!(input as PlaceIntoInput);
    let ptr = inp.ptr;

    let struct_expr = match inp.expr {
        syn::Expr::Struct(s) => s,
        e => {

            let generated = inner_place_expr(quote!{ (*ptr) }, &e, 0);
            return quote! {

                {
                    let _ensure_correct = || { #e };

                    let ptr = #ptr;

                    #generated
                }
            }.into()
        }
    };

    if let Some(rest) = &struct_expr.rest {
        return syn::Error::new_spanned(rest, "place_boxed limitation: Cannot use \
        ..Rest expressions in struct initializers")
            .to_compile_error()
            .into();
    }




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
            let _ensure_struct_correct = || {
                #struct_expr
            };

            let ptr = #ptr;

            #(#generated)*
        }
    }.into()
}

