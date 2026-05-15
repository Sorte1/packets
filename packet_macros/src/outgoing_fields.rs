use proc_macro2::TokenStream;
use quote::quote;
use syn::{Attribute, Data, DeriveInput, Fields};

#[derive(Debug, Default)]
struct FieldOptions {
    nested: bool,
    skip: bool,
    coerce: Option<crate::coerce::CoerceSpec>,
}

pub fn expand_outgoing_fields(input: DeriveInput) -> syn::Result<TokenStream> {
    let struct_name = &input.ident;
    let generics = &input.generics;

    if !generics.params.is_empty() {
        return Err(syn::Error::new_spanned(
            generics,
            "OutgoingFields derive does not yet support generic structs",
        ));
    }

    let fields_named = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => fields,
            Fields::Unnamed(_) => {
                return Err(syn::Error::new_spanned(
                    &input.ident,
                    "OutgoingFields derive does not support tuple structs",
                ))
            }
            Fields::Unit => {
                return Err(syn::Error::new_spanned(
                    &input.ident,
                    "OutgoingFields derive does not support unit structs",
                ))
            }
        },
        _ => {
            return Err(syn::Error::new_spanned(
                &input.ident,
                "OutgoingFields derive can only be used on structs",
            ))
        }
    };

    let mut field_pushes = Vec::new();

    for field in &fields_named.named {
        let field_ident = field.ident.as_ref().ok_or_else(|| {
            syn::Error::new_spanned(field, "OutgoingFields derive requires named fields")
        })?;

        let field_options = parse_field_options(&field.attrs)?;

        if field_options.skip {
            continue;
        }

        if field_options.nested {
            field_pushes.push(quote! {
                values.push(serde_value::Value::Seq(
                    packet_core::OutgoingFields::to_field_values(&self.#field_ident)?
                ));
            });
        } else if let Some(spec) = &field_options.coerce {
            let flags = crate::coerce::coerce_flags_tokens(spec);
            field_pushes.push(quote! {
                values.push(packet_core::decode::encode_field_coerce(&self.#field_ident, #flags)?);
            });
        } else {
            field_pushes.push(quote! {
                values.push(serde_value::to_value(&self.#field_ident)?);
            });
        }
    }

    Ok(quote! {
        impl packet_core::OutgoingFields for #struct_name {
            fn to_field_values(&self) -> anyhow::Result<Vec<serde_value::Value>> {
                let mut values = Vec::new();

                #(#field_pushes)*

                Ok(values)
            }
        }
    })
}

fn parse_field_options(attrs: &[Attribute]) -> syn::Result<FieldOptions> {
    let mut options = FieldOptions::default();

    for attr in attrs {
        if !attr.path().is_ident("packet") {
            continue;
        }

        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("nested") {
                options.nested = true;
                Ok(())
            } else if meta.path.is_ident("skip") {
                options.skip = true;
                Ok(())
            } else if meta.path.is_ident("coerce") {
                options.coerce = Some(crate::coerce::parse_coerce_meta(&meta)?);
                Ok(())
            } else {
                Err(meta.error(
                    "unsupported #[packet(...)] field option for OutgoingFields; supported: nested, skip, coerce",
                ))
            }
        })?;
    }

    Ok(options)
}
