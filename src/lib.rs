extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Expr};

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
            match &field.expr {
                Expr::Repeat(r) => {
                    let len = &r.len;
                    let val = &r.expr;

                    quote! {
                        for i in 0..#len {
                            (&raw mut (*ptr).#name[i]).write(#val);
                        }
                    }
                },
                Expr::Array(r) => {
                    let idxs = 0..r.elems.len();
                    let vals = &r.elems;
                    quote! {
                        #((&raw mut (*ptr).#name[#idxs]).write(#vals);)*
                    }
                }
                _ => {
                    let v = &field.expr;
                    quote!{
                        (&raw mut (*ptr).#name).write(#v);
                    }
                }
            }
        )
    }




    quote! {
        {
            let mut res = std::boxed::Box::<#path>::new_uninit();
            unsafe {
                let ptr = res.as_mut_ptr();

                #(#generated)*

                res.assume_init()
            }
        }
    }.into()
}


