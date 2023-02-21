use proc_macro::TokenStream;
use proc_macro_error::{abort, abort_if_dirty};
use syn::{parse_macro_input, DeriveInput, Data, Fields, Ident};

#[proc_macro_derive(Varying)]
#[proc_macro_error::proc_macro_error]
pub fn varying(input: TokenStream) -> TokenStream
{
    let input = parse_macro_input!(input as DeriveInput);
    let struct_name = input.ident;

    let fields_name: Vec<Ident> = if let Data::Struct(ds) = input.data
    {
        if let Fields::Named(field) = ds.fields
        {
            field.named.into_iter().map(|f|
            {
                f.ident.unwrap()
            }).collect()
        }
        else
        {
            abort!(struct_name, "Varying 的成员必须有名称")
        }
    }
    else
    {
        abort!(struct_name, "Varying 必须是一个结构体")
    };

    abort_if_dirty();
    
    quote::quote!
    {
        impl std::ops::Add<#struct_name> for #struct_name
        {
            type Output = #struct_name;

            fn add(self, rhs: Self) -> Self::Output
            {
                Self {
                    #( #fields_name: self.#fields_name + rhs.#fields_name, )*
                }
            }
        }

        impl std::ops::Mul<f32> for #struct_name
        {
            type Output = #struct_name;

            fn mul(self, rhs: f32) -> Self::Output
            {
                Self {
                    #( #fields_name: self.#fields_name * rhs, )*
                }
            }
        }

        impl Varying for #struct_name {}
    }.into()
}