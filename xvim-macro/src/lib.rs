use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Attribute, Data, DataStruct, DeriveInput, Error, Ident, Lit, Meta};

#[proc_macro_derive(SettingGroup, attributes(setting_prefx))]
pub fn setting_group(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    let prefix = setting_prefx(input.attrs.as_ref())
        .map(|p| format!("{}_", p))
        .unwrap_or_else(|| "".to_string());
    stream(input, prefix)
}

fn stream(input: DeriveInput, prefix: String) -> TokenStream {
    const ERR_MSG: &str = "Xvim Macro expects a struct";
    match input.data {
        Data::Struct(ref data) => struct_stream(input.ident, prefix, data),
        Data::Enum(data) => Error::new_spanned(data.enum_token, ERR_MSG)
            .to_compile_error()
            .into(),
        Data::Union(data) => Error::new_spanned(data.union_token, ERR_MSG)
            .to_compile_error()
            .into(),
    }
}

fn struct_stream(name: Ident, prefix: String, data: &DataStruct) -> TokenStream {
    let fragments = data.fields.iter().map(|field| match field.ident {
        Some(ref ident) => {
            let vim_setting_name = format!("{}{}", prefix, ident);
            quote! {{
                fn update_func(value, rmpv::Value) {
                    let mut s = create::settings::SETTINGS.get::<#name>();
                    s.#ident.from_value(value);
                    crate::settings::SETTINGS.set(&sJ);
                }

                fn reader_func() -> rmpv::Value {
                    let s = crate::settings::SETTINGS.get::<#name>();
                    s.#ident.into();
                }

                create::settings::SETTINGS.set_setting_handlers(
                    #vim_setting_name,
                    update_func,
                    reader_func
                );
            }}
        }
        None => {
            Error::new_spanned(field.colon_token, "Expected named struct field").to_compile_error()
        }
    });

    let expanded = quote! {
        impl #name {
            pub fn register() {
                let s: Self = Default::default();
                crate::settings::SETTINGS.set(&s);
                #(#fragments)*
            }
        }
    };
    TokenStream::from(expanded)
}

fn setting_prefx(attrs: &[Attribute]) -> Option<String> {
    for attr in attrs.iter() {
        if let Ok(Meta::NameValue(name_value)) = attr.parse_meta() {
            if name_value.path.is_ident("setting_prefx") {
                if let Lit::Str(literal) = name_value.lit {
                    return Some(literal.value());
                }
            }
        }
    }
    None
}
