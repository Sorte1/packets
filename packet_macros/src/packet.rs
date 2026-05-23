use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    Attribute, Data, DeriveInput, Fields, GenericArgument, LitInt, LitStr, PathArguments, Type,
};

#[derive(Debug, Default)]
struct PacketOptions {
    event: Option<String>,
    allow_extra: bool,
    default_missing: bool,
    scalar_as_seq: bool,
}

#[derive(Debug, Default)]
struct FieldOptions {
    default: bool,
    coerce: Option<crate::coerce::CoerceSpec>,
    chunks: Option<(Type, LitInt, bool)>,
    nested: bool,
    skip: bool,
    extras: bool,
    at: Option<usize>,
}

pub fn expand_packet(input: DeriveInput) -> syn::Result<TokenStream> {
    let struct_name = &input.ident;
    let generics = &input.generics;

    if !generics.params.is_empty() {
        return Err(syn::Error::new_spanned(
            generics,
            "Packet derive does not yet support generic structs",
        ));
    }

    let packet_options = parse_packet_options(&input.attrs)?;
    let event_name = packet_options.event.clone().ok_or_else(|| {
        syn::Error::new_spanned(&input.ident, "missing #[packet(event = \"...\")] on struct")
    })?;

    let fields_named = extract_named_fields(&input)?;

    let mut decode_statements = Vec::new();
    let mut init_fields = Vec::new();
    let mut outgoing_statements = Vec::new();
    let mut has_extras = false;
    let mut next_index: usize = 0;

    for field in fields_named.named.iter() {
        let field_ident = field
            .ident
            .as_ref()
            .ok_or_else(|| syn::Error::new_spanned(field, "Packet derive requires named fields"))?;
        let field_ty = &field.ty;
        let field_name_string = field_ident.to_string();
        let field_options = parse_field_options(&field.attrs)?;

        let index = if let Some(explicit) = field_options.at {
            if explicit < next_index {
                return Err(syn::Error::new_spanned(
                    field,
                    format!(
                        "#[packet(at = {explicit})] is below the next available index {next_index} (indices must be monotonic)"
                    ),
                ));
            }
            explicit
        } else {
            next_index
        };
        next_index = index + 1;

        if field_options.extras {
            if has_extras {
                return Err(syn::Error::new_spanned(
                    field,
                    "only one #[packet(extras)] field is allowed",
                ));
            }
            has_extras = true;

            decode_statements.push(quote! {
                let #field_ident: Vec<serde_value::Value> =
                    payload.iter().skip(#index).cloned().collect();
            });
            init_fields.push(quote! { #field_ident });
            outgoing_statements.push(quote! {
                values.extend(self.#field_ident.iter().cloned());
            });
            continue;
        }

        let decode_expr = if let Some((item_ty, chunk_size, zero_as_empty)) = &field_options.chunks {
            let fn_name = if *zero_as_empty {
                quote! { packet_core::decode::decode_chunks_zero_as_empty }
            } else {
                quote! { packet_core::decode::decode_chunks }
            };
            quote! {
                #fn_name::<#item_ty>(
                    #event_name,
                    #index,
                    #field_name_string,
                    #chunk_size,
                    payload.get(#index),
                )?
            }
        } else if let Some(spec) = &field_options.coerce {
            let flags = crate::coerce::coerce_flags_tokens(spec);
            quote! {
                packet_core::decode::decode_field_coerce::<#field_ty>(
                    #event_name,
                    #index,
                    #field_name_string,
                    #flags,
                    payload.get(#index),
                )?
            }
        } else if let Some(inner_ty) = extract_option_inner(field_ty) {
            quote! {
                packet_core::decode::decode_optional_field::<#inner_ty>(
                    #event_name,
                    #index,
                    #field_name_string,
                    payload.get(#index),
                )?
            }
        } else if field_options.default || packet_options.default_missing {
            quote! {
                packet_core::decode::decode_field_with_default::<#field_ty>(
                    #event_name,
                    #index,
                    #field_name_string,
                    payload.get(#index),
                )?
            }
        } else {
            quote! {
                packet_core::decode::decode_field::<#field_ty>(
                    #event_name,
                    #index,
                    #field_name_string,
                    payload.get(#index),
                )?
            }
        };

        decode_statements.push(quote! {
            let #field_ident = #decode_expr;
        });

        init_fields.push(quote! {
            #field_ident
        });

        if !field_options.skip {
            let wire_index = index + 1;
            outgoing_statements.push(quote! {
                while values.len() < #wire_index + 1 {
                    values.push(serde_value::Value::Unit);
                }
            });
            if field_options.nested {
                outgoing_statements.push(quote! {
                    values[#wire_index] = serde_value::Value::Seq(
                        packet_core::OutgoingFields::to_field_values(&self.#field_ident)?
                    );
                });
            } else if let Some(spec) = &field_options.coerce {
                let flags = crate::coerce::coerce_flags_tokens(spec);
                outgoing_statements.push(quote! {
                    values[#wire_index] = packet_core::decode::encode_field_coerce(&self.#field_ident, #flags)?;
                });
            } else {
                outgoing_statements.push(quote! {
                    values[#wire_index] = serde_value::to_value(&self.#field_ident)?;
                });
            }
        }
    }

    let field_count = next_index;
    let wrap_scalar = packet_options.scalar_as_seq;

    if has_extras && packet_options.allow_extra {
        return Err(syn::Error::new_spanned(
            &input.ident,
            "#[packet(allow_extra)] is redundant when an #[packet(extras)] field is present; remove allow_extra",
        ));
    }

    let extra_field_check = if packet_options.allow_extra || has_extras {
        quote! {}
    } else {
        quote! {
            if payload.len() > #field_count {
                anyhow::bail!(
                    "event {:?} had {} payload values, expected at most {}",
                    #event_name,
                    payload.len(),
                    #field_count
                );
            }
        }
    };

    Ok(quote! {
        impl packet_core::PacketMeta for #struct_name {
            const EVENT_NAME: &'static str = #event_name;
        }

        impl packet_core::PacketDecode for #struct_name {
            fn decode_payload(payload: &[serde_value::Value]) -> anyhow::Result<Self> {
                let payload = packet_core::decode::seq_payload(payload, #wrap_scalar);

                #extra_field_check

                #(#decode_statements)*

                Ok(Self {
                    #(#init_fields),*
                })
            }
        }

        impl packet_core::OutgoingPacket for #struct_name {
            fn to_values(&self) -> anyhow::Result<Vec<serde_value::Value>> {
                let mut values = Vec::new();

                values.push(serde_value::Value::String(
                    <Self as packet_core::PacketMeta>::EVENT_NAME.to_string()
                ));

                #(#outgoing_statements)*

                Ok(values)
            }
        }
    })
}

fn extract_named_fields(input: &DeriveInput) -> syn::Result<&syn::FieldsNamed> {
    match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => Ok(fields),
            Fields::Unnamed(_) => Err(syn::Error::new_spanned(
                &input.ident,
                "Packet derive does not support tuple structs",
            )),
            Fields::Unit => Err(syn::Error::new_spanned(
                &input.ident,
                "Packet derive does not support unit structs",
            )),
        },
        Data::Enum(_) => Err(syn::Error::new_spanned(
            &input.ident,
            "Packet derive can only be used on structs",
        )),
        Data::Union(_) => Err(syn::Error::new_spanned(
            &input.ident,
            "Packet derive cannot be used on unions",
        )),
    }
}

fn parse_packet_options(attrs: &[Attribute]) -> syn::Result<PacketOptions> {
    let mut options = PacketOptions::default();

    for attr in attrs {
        if !attr.path().is_ident("packet") {
            continue;
        }

        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("event") {
                if options.event.is_some() {
                    return Err(meta.error("duplicate packet event option"));
                }
                let value = meta.value()?;
                let lit: LitStr = value.parse()?;
                options.event = Some(lit.value());
                Ok(())
            } else if meta.path.is_ident("allow_extra") {
                options.allow_extra = true;
                Ok(())
            } else if meta.path.is_ident("default_missing") {
                options.default_missing = true;
                Ok(())
            } else if meta.path.is_ident("scalar_as_seq") {
                options.scalar_as_seq = true;
                Ok(())
            } else {
                Err(meta.error(
                    "unsupported #[packet(...)] struct option; supported: event, allow_extra, default_missing, scalar_as_seq",
                ))
            }
        })?;
    }

    Ok(options)
}

fn parse_field_options(attrs: &[Attribute]) -> syn::Result<FieldOptions> {
    let mut options = FieldOptions::default();

    for attr in attrs {
        if !attr.path().is_ident("packet") {
            continue;
        }

        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("default") {
                options.default = true;
                Ok(())
            } else if meta.path.is_ident("coerce") {
                options.coerce = Some(crate::coerce::parse_coerce_meta(&meta)?);
                Ok(())
            } else if meta.path.is_ident("chunks") {
                let content;
                syn::parenthesized!(content in meta.input);
                let item_ty: Type = content.parse()?;
                let _comma: syn::Token![,] = content.parse()?;
                let chunk_size: LitInt = content.parse()?;
                let zero_as_empty = if content.peek(syn::Token![,]) {
                    let _comma: syn::Token![,] = content.parse()?;
                    let ident: syn::Ident = content.parse()?;
                    if ident != "zero_as_empty" {
                        return Err(syn::Error::new(
                            ident.span(),
                            "unknown chunks option; supported: zero_as_empty",
                        ));
                    }
                    true
                } else {
                    false
                };
                options.chunks = Some((item_ty, chunk_size, zero_as_empty));
                Ok(())
            } else if meta.path.is_ident("nested") {
                options.nested = true;
                Ok(())
            } else if meta.path.is_ident("skip") {
                options.skip = true;
                Ok(())
            } else if meta.path.is_ident("extras") {
                options.extras = true;
                Ok(())
            } else if meta.path.is_ident("at") {
                let value = meta.value()?;
                let lit: LitInt = value.parse()?;
                let n: usize = lit.base10_parse()?;
                options.at = Some(n);
                Ok(())
            } else {
                Err(meta.error(
                    "unsupported #[packet(...)] field option; supported: default, coerce, chunks(Type, N), nested, skip, extras, at = N",
                ))
            }
        })?;
    }

    Ok(options)
}

fn extract_option_inner(ty: &Type) -> Option<&Type> {
    let Type::Path(type_path) = ty else {
        return None;
    };

    let last_segment = type_path.path.segments.last()?;
    if last_segment.ident != "Option" {
        return None;
    }

    let PathArguments::AngleBracketed(angle_args) = &last_segment.arguments else {
        return None;
    };

    let first_arg = angle_args.args.first()?;
    match first_arg {
        GenericArgument::Type(inner_ty) => Some(inner_ty),
        _ => None,
    }
}
