use proc_macro::TokenStream;
use syn::parse_macro_input;

mod coerce;
mod event_enum;
mod outgoing_event_enum;
mod outgoing_fields;
mod packet;

#[proc_macro_derive(Packet, attributes(packet))]
pub fn derive_packet(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as syn::DeriveInput);

    match packet::expand_packet(input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

#[proc_macro_derive(OutgoingFields, attributes(packet))]
pub fn derive_outgoing_fields(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as syn::DeriveInput);

    match outgoing_fields::expand_outgoing_fields(input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

#[proc_macro_derive(OutgoingEventEnum, attributes(event))]
pub fn derive_outgoing_event_enum(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as syn::DeriveInput);

    match outgoing_event_enum::expand_outgoing_event_enum(input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

#[proc_macro_derive(EventEnum, attributes(event))]
pub fn derive_event_enum(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as syn::DeriveInput);

    match event_enum::expand_event_enum(input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}
