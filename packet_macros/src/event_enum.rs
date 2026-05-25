use proc_macro2::TokenStream;
use quote::quote;
use syn::{Attribute, Data, DeriveInput, Fields, Type};

#[derive(Debug, Default)]
struct VariantOptions {
    unknown: bool,
}

pub fn expand_event_enum(input: DeriveInput) -> syn::Result<TokenStream> {
    let enum_name = &input.ident;
    let generics = &input.generics;

    if !generics.params.is_empty() {
        return Err(syn::Error::new_spanned(
            generics,
            "EventEnum derive does not yet support generic enums",
        ));
    }

    let enum_data = match &input.data {
        Data::Enum(data) => data,
        _ => {
            return Err(syn::Error::new_spanned(
                &input.ident,
                "EventEnum can only be derived for enums",
            ))
        }
    };

    let mut decode_arms = Vec::new();
    let mut encode_arms = Vec::new();
    let mut unknown_decode_arm = None::<TokenStream>;
    let mut unknown_encode_arm = None::<TokenStream>;
    let mut found_unknown = false;

    for variant in &enum_data.variants {
        let variant_ident = &variant.ident;
        let opts = parse_variant_options(&variant.attrs)?;

        if opts.unknown {
            if found_unknown {
                return Err(syn::Error::new_spanned(
                    variant,
                    "only one #[event(unknown)] variant is allowed",
                ));
            }
            found_unknown = true;

            validate_unknown_variant(variant)?;

            unknown_decode_arm = Some(quote! {
                _ => Ok(Self::#variant_ident(event_name.to_string(), payload.to_vec())),
            });

            unknown_encode_arm = Some(quote! {
                Self::#variant_ident(name, payload) => {
                    let mut values = vec![serde_value::Value::String(name.clone())];
                    values.extend(payload.iter().cloned());
                    Ok(values)
                }
            });

            continue;
        }

        let payload_ty = extract_single_unnamed_field_type(variant)?;

        decode_arms.push(quote! {
            x if x == <#payload_ty as packet_core::PacketMeta>::EVENT_NAME => {
                <#payload_ty as packet_core::PacketDecode>::decode_payload(payload)
                    .map(Self::#variant_ident)
                    .map_err(|err| {
                        anyhow::anyhow!(
                            "Failed to decode event {:?}: {}. Payload: {:?}",
                            event_name,
                            err,
                            payload
                        )
                    })
            }
        });

        encode_arms.push(quote! {
            Self::#variant_ident(p) => packet_core::OutgoingPacket::to_values(p),
        });
    }

    let unknown_decode_arm = if let Some(arm) = unknown_decode_arm {
        arm
    } else {
        quote! {
            _ => anyhow::bail!("Unknown event {:?} with payload {:?}", event_name, payload),
        }
    };
    let unknown_encode_arm = unknown_encode_arm.unwrap_or_default();

    Ok(quote! {
        impl #enum_name {
            pub fn from_event_name(
                event_name: &str,
                payload: &[serde_value::Value],
            ) -> anyhow::Result<Self> {
                match event_name {
                    #(#decode_arms,)*
                    #unknown_decode_arm
                }
            }
        }

        impl packet_core::OutgoingPacket for #enum_name {
            fn to_values(&self) -> anyhow::Result<Vec<serde_value::Value>> {
                match self {
                    #(#encode_arms)*
                    #unknown_encode_arm
                }
            }
        }
    })
}

fn parse_variant_options(attrs: &[Attribute]) -> syn::Result<VariantOptions> {
    let mut options = VariantOptions::default();

    for attr in attrs {
        if !attr.path().is_ident("event") {
            continue;
        }

        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("unknown") {
                options.unknown = true;
                Ok(())
            } else {
                Err(meta.error("unsupported #[event(...)] option; supported: unknown"))
            }
        })?;
    }

    Ok(options)
}

fn extract_single_unnamed_field_type(variant: &syn::Variant) -> syn::Result<&Type> {
    match &variant.fields {
        Fields::Unnamed(fields) if fields.unnamed.len() == 1 => {
            Ok(&fields.unnamed.first().unwrap().ty)
        }
        Fields::Unnamed(_) => Err(syn::Error::new_spanned(
            variant,
            "normal EventEnum variants must have exactly one unnamed field, like Variant(PacketType)",
        )),
        Fields::Named(_) => Err(syn::Error::new_spanned(
            variant,
            "EventEnum does not support named-field enum variants",
        )),
        Fields::Unit => Err(syn::Error::new_spanned(
            variant,
            "EventEnum does not support unit variants for normal event cases",
        )),
    }
}

fn validate_unknown_variant(variant: &syn::Variant) -> syn::Result<()> {
    match &variant.fields {
        Fields::Unnamed(fields) if fields.unnamed.len() == 2 => Ok(()),
        _ => Err(syn::Error::new_spanned(
            variant,
            "#[event(unknown)] variant must look like Unknown(String, Vec<serde_value::Value>)",
        )),
    }
}
