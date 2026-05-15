use proc_macro2::TokenStream;
use quote::quote;
use syn::{Attribute, Data, DeriveInput, Fields, Type};

#[derive(Debug, Default)]
struct VariantOptions {
    unknown: bool,
}

pub fn expand_outgoing_event_enum(input: DeriveInput) -> syn::Result<TokenStream> {
    let enum_name = &input.ident;
    let generics = &input.generics;

    if !generics.params.is_empty() {
        return Err(syn::Error::new_spanned(
            generics,
            "OutgoingEventEnum derive does not yet support generic enums",
        ));
    }

    let enum_data = match &input.data {
        Data::Enum(data) => data,
        _ => {
            return Err(syn::Error::new_spanned(
                &input.ident,
                "OutgoingEventEnum can only be derived for enums",
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

            match &variant.fields {
                Fields::Unnamed(fields) if fields.unnamed.len() == 1 => {}
                _ => {
                    return Err(syn::Error::new_spanned(
                        variant,
                        "#[event(unknown)] variant must have exactly one field: Unknown(Vec<u8>)",
                    ))
                }
            }

            unknown_decode_arm = Some(quote! {
                #variant_ident
            });

            unknown_encode_arm = Some(quote! {
                Self::#variant_ident(raw) => Ok(raw.clone()),
            });

            continue;
        }

        let payload_ty = extract_single_unnamed_field_type(variant)?;

        decode_arms.push(quote! {
            x if x == <#payload_ty as packet_core::PacketMeta>::EVENT_NAME => {
                Some(
                    <#payload_ty as packet_core::PacketDecode>::decode_payload(payload)
                        .map(Self::#variant_ident)
                        .map_err(|err| anyhow::anyhow!(
                            "Failed to decode outgoing event {:?}: {}. Payload: {:?}",
                            event_name, err, payload
                        ))
                )
            }
        });

        encode_arms.push(quote! {
            Self::#variant_ident(p) => {
                let values = packet_core::OutgoingPacket::to_values(p)?;
                packet_core::wire::repack_frame(&values, trailer)
            }
        });
    }

    let passthrough_variant = unknown_decode_arm.unwrap_or_else(|| {
        quote! { compile_error!("OutgoingEventEnum requires a #[event(unknown)] variant") }
    });
    let unknown_encode_arm = unknown_encode_arm.unwrap_or_else(TokenStream::new);

    Ok(quote! {
        impl packet_core::OutgoingEventEnum for #enum_name {
            fn try_from_event(
                event_name: &str,
                payload: &[serde_value::Value],
            ) -> Option<anyhow::Result<Self>> {
                match event_name {
                    #(#decode_arms,)*
                    _ => None,
                }
            }

            fn passthrough(raw: Vec<u8>) -> Self {
                Self::#passthrough_variant(raw)
            }

            fn to_frame(&self, trailer: [u8; 2]) -> anyhow::Result<Vec<u8>> {
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
        _ => Err(syn::Error::new_spanned(
            variant,
            "OutgoingEventEnum variants must have exactly one unnamed field",
        )),
    }
}
