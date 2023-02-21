use proc_macro::TokenStream;
use proc_macro_error::{abort, abort_if_dirty};
use quote::ToTokens;
use syn::{Fields, Data, parse_macro_input, DeriveInput, Type};

#[proc_macro_derive(Shader, attributes(attribute, sampler, varying, uniform))]
#[proc_macro_error::proc_macro_error]
pub fn shader(input: TokenStream) -> TokenStream
{
    let input = parse_macro_input!(input as DeriveInput);
    let struct_name = input.ident;

    let mut varying_field = None;

    let mut attribute_fields = Vec::new();
    let mut sampler_fields = Vec::new();
    let mut clone_fields = Vec::new();

    if let Data::Struct(ds) = input.data
    {
        if let Fields::Named(fields) = ds.fields
        {
            fields.named.into_iter().for_each(|f|
            {
                f.attrs.iter().for_each(|attr|
                {
                    let path = attr.path.to_token_stream().to_string();

                    match path.as_str()
                    {
                        "attribute" =>
                        {
                            attribute_fields.push(f.ident.clone().unwrap());
                        }

                        "varying" =>
                        {
                            if varying_field.is_none()
                            {
                                varying_field = Some(f.clone());
                            }
                            else
                            {
                                abort!(struct_name, "Shader 拥有多于一个 Varying");
                            }
                        }

                        "sampler" =>
                        {
                            sampler_fields.push(f.ident.clone().unwrap());
                            clone_fields.push(f.ident.clone().unwrap());
                        }

                        "uniform" =>
                        {
                            clone_fields.push(f.ident.clone().unwrap());
                        }

                        _ => ()
                    }
                });
            })
        }
        else
        {
            abort!(struct_name, "Shader 的字段必须有名字");
        }
    }
    else
    {
        abort!(struct_name, "Shader 必须是结构体");
    };
    
    abort_if_dirty();

    let varying_field_type = varying_field.as_ref().unwrap().attrs[0].parse_args::<Type>().unwrap();
    let varying_field_name = varying_field.unwrap().ident.unwrap();

    quote::quote!
    {
        impl Shader<#varying_field_type> for #struct_name
        {
            fn next(&mut self)
            {
                #(self.#attribute_fields.forward();)*
            }

            fn reset(&mut self)
            {
                #(self.#attribute_fields.reset();)*
                self.#varying_field_name.clear();
            }

            fn get_varying(&self) -> &Vec<#varying_field_type>
            {
                &self.#varying_field_name
            }

            fn compute_level(&mut self, sample_point: i32)
            {
                #(self.#sampler_fields.compute_level(sample_point);)*
            }
        }

        impl Clone for #struct_name
        {
            fn clone(&self) -> Self
            {
                Self {
                    #(#clone_fields: self.#clone_fields.clone(),)*
                    ..Default::default()
                }
            }
        }
    }.into()
}
