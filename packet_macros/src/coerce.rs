use proc_macro2::TokenStream;
use quote::quote;
use syn::meta::ParseNestedMeta;

#[derive(Debug, Default)]
pub struct CoerceSpec {
    pub num_to_bool: bool,
    pub bool_to_num: bool,
    pub str_num: bool,
}

pub fn parse_coerce_meta(meta: &ParseNestedMeta) -> syn::Result<CoerceSpec> {
    if meta.input.peek(syn::token::Paren) {
        let content;
        syn::parenthesized!(content in meta.input);
        let mut spec = CoerceSpec::default();
        loop {
            let ident: syn::Ident = content.parse()?;
            match ident.to_string().as_str() {
                "num_to_bool" => spec.num_to_bool = true,
                "bool_to_num" => spec.bool_to_num = true,
                "str_num" => spec.str_num = true,
                other => {
                    return Err(syn::Error::new(
                        ident.span(),
                        format!(
                            "unknown coerce strategy {other:?}; supported: num_to_bool, bool_to_num, str_num"
                        ),
                    ))
                }
            }
            if content.is_empty() {
                break;
            }
            let _comma: syn::Token![,] = content.parse()?;
            if content.is_empty() {
                break;
            }
        }
        Ok(spec)
    } else {
        Ok(CoerceSpec {
            num_to_bool: true,
            ..CoerceSpec::default()
        })
    }
}

pub fn coerce_flags_tokens(spec: &CoerceSpec) -> TokenStream {
    let num_to_bool = spec.num_to_bool;
    let bool_to_num = spec.bool_to_num;
    let str_num = spec.str_num;
    quote! {
        packet_core::decode::CoerceFlags {
            num_to_bool: #num_to_bool,
            bool_to_num: #bool_to_num,
            str_num: #str_num,
        }
    }
}
