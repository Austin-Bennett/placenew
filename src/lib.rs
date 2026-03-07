extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Expr};

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


