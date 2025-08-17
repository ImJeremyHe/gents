use case::{convert_camel_from_pascal, convert_camel_from_snake};
use proc_macro2::Span;
use syn::{parse_macro_input, DeriveInput};
mod case;
mod container;
mod serde_json;
mod symbol;

use container::{Container, RenameAll};
use proc_macro::TokenStream;
use quote::quote;

use crate::container::GentsWasmAttrs;

#[proc_macro_attribute]
pub fn gents_header(attr: TokenStream, item: TokenStream) -> TokenStream {
    let item: proc_macro2::TokenStream = item.into();
    let attrs = syn::parse2::<GentsWasmAttrs>(attr.into()).expect("parse error, please check");
    let file_name = attrs.get_file_name();
    quote! {
        #[derive(::serde::Serialize, ::serde::Deserialize)]
        #[cfg_attr(feature = "gents", derive(::gents_derives::TS))]
        #[cfg_attr(
            any(test, feature = "gents"),
            ts(file_name = #file_name, rename_all = "camelCase")
        )]
        #[serde(rename_all = "camelCase")]
        #item
    }
    .into()
}

#[proc_macro_derive(TS, attributes(ts))]
pub fn derive_ts(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let container = Container::from_ast(&input);
    let impl_block = get_impl_block(container.clone());
    let serde = serde_json::get_serde_impl_block(container, &input);
    quote! {
        #impl_block
        #serde
    }
    .into()
}

fn get_impl_block(container: Container) -> proc_macro2::TokenStream {
    let file_name = container.file_name;
    let is_enum = container.is_enum;
    let rename_all = container.rename_all;
    let ident = container.ident;
    let fields = container.fields;
    let rename = container.rename;
    let ts_name = match rename {
        Some(s) if is_enum => s,
        _ => ident.to_string(),
    };
    let comments = container.comments;
    let need_builder = container.need_builder;
    let tag = if let Some(t) = container.tag {
        t
    } else {
        "".to_string()
    };
    let register_func = {
        let generics_dep_register = container.generics.iter().map(|g| {
            quote! {
                <#g as ::gents::TS>::_register(manager, true);
            }
        });
        let generic_register = if !container.generics.is_empty() {
            let placehoders = container
                .generics
                .iter()
                .map(|g| syn::Ident::new(&format!("{}_{}", ident, g), Span::call_site()));
            let _placeholders = placehoders.clone();
            let placeholders_str = container
                .generics
                .iter()
                .map(|g| g.to_string())
                .collect::<Vec<_>>()
                .join(", ");
            quote! {
                let _d = <#ident<#(#placehoders),*> as ::gents::TS>::_register(manager, false);
                deps.push(_d);
                manager.add_generics_map(_d, #placeholders_str.to_string());
                generic = Some(_d);
            }
        } else {
            quote! {}
        };
        let field_ds = fields.into_iter().filter(|f| !f.skip).map(|s| {
            let fi = s.ident;
            let rename = s.rename;
            let ty = s.ty;
            let field_comments = s.comments;
            let tag_value = if let Some(v) = s.tag_value {
                v
            } else {
                "".to_string()
            };
            let name = match (rename, &rename_all) {
                (None, None) => fi.to_string(),
                (None, Some(RenameAll::CamelCase)) => {
                    let s = fi.to_string();
                    if is_enum {
                        convert_camel_from_pascal(s)
                    } else {
                        convert_camel_from_snake(s)
                    }
                }
                (Some(s), _) => s,
            };
            if let Some(ty) = ty {
                quote! {
                    let dep = <#ty as ::gents::TS>::_register(manager, true);
                    deps.push(dep);
                    let fd = ::gents::FieldDescriptor {
                        ident: #name.to_string(),
                        optional: <#ty as ::gents::TS>::_is_optional(),
                        ts_ty: <#ty as ::gents::TS>::_ts_name(),
                        comments: vec![#(#field_comments.to_string()),*],
                        tag_value: #tag_value.to_string(),
                    };
                    fields.push(fd);
                }
            } else {
                quote! {
                    let fd = ::gents::FieldDescriptor {
                        ident: #name.to_string(),
                        optional: false,
                        ts_ty: String::from(""),
                        comments: vec![#(#field_comments.to_string()),*],
                        tag_value: #tag_value.to_string(),
                    };
                    fields.push(fd);
                }
            }
        });
        let descriptor = if is_enum {
            quote! {
                let _enum = ::gents::EnumDescriptor {
                    dependencies: deps,
                    fields,
                    file_name: #file_name.to_string(),
                    ts_name: #ts_name.to_string(),
                    comments: vec![#(#comments.to_string()),*],
                    tag: #tag.to_string(),
                    generic,
                };
                let descriptor = ::gents::Descriptor::Enum(_enum);
            }
        } else {
            quote! {
                let _interface = ::gents::InterfaceDescriptor {
                    dependencies: deps,
                    fields,
                    file_name: #file_name.to_string(),
                    ts_name: #ts_name.to_string(),
                    comments: vec![#(#comments.to_string()),*],
                    need_builder: #need_builder,
                    generic,
                };
                let descriptor = ::gents::Descriptor::Interface(_interface);
            }
        };
        quote! {
            fn _register(manager: &mut ::gents::DescriptorManager, generic_base: bool) -> usize {
                let type_id = std::any::TypeId::of::<Self>();
                let mut deps = ::std::vec::Vec::<usize>::new();
                let mut fields = ::std::vec::Vec::<::gents::FieldDescriptor>::new();
                let mut generic = None;
                if generic_base {
                    #generic_register
                }
                #(#field_ds)*
                #(#generics_dep_register)*
                #descriptor
                manager.registry(type_id, descriptor)
            }
        }
    };
    let generics = container
        .generics
        .iter()
        .map(|g| quote! {generics_names.push(<#g as ::gents::TS>::_ts_name())});
    let ts_name_func = if container.generics.is_empty() {
        quote! {
            fn _ts_name() -> String {
                #ts_name.to_string()
            }
        }
    } else {
        quote! {
            fn _ts_name() -> String {
                let name = #ts_name;
                let mut generics_names = Vec::<String>::new();
                #(#generics;)*
                let generics = generics_names.join(", ");
                format!("{}<{}>", name, generics)
            }
        }
    };
    if container.generics.is_empty() {
        quote! {
            #[cfg(any(test, feature="gents"))]
            impl ::gents::TS for #ident {
                #register_func
                #ts_name_func
            }
        }
    } else {
        let generics_ts = container.generics.iter().map(|g| {
            quote! {
                #g: ::gents::TS + Clone + 'static
            }
        });
        let generics_idents = &container.generics;
        let placeholder_impls = generics_idents
            .iter()
            .map(|g| get_generic_placeholder(&ident, g));
        quote! {
            #(#placeholder_impls)*
            #[cfg(any(test, feature="gents"))]
            impl<#(#generics_ts),*>
            ::gents::TS for #ident<#(#generics_idents),*>{
                #register_func
                #ts_name_func
            }
        }
    }
}

fn get_generic_placeholder(
    parent_ident: &syn::Ident,
    placeholder: &syn::Ident,
) -> proc_macro2::TokenStream {
    let tag_ident = syn::Ident::new(
        &format!("{}_{}", parent_ident, placeholder),
        Span::call_site(),
    );
    let ts_name = format!("{}", placeholder);
    quote! {
        #[cfg(any(test, feature="gents"))]
        #[derive(Clone)]
        #[cfg(any(test, feature="gents"))]
        struct #tag_ident;
        #[cfg(any(test, feature="gents"))]
        impl ::gents::TS for #tag_ident {
            fn _register(manager: &mut ::gents::DescriptorManager, _generic_base: bool) -> usize {
                let type_id = std::any::TypeId::of::<Self>();
                let descriptor = ::gents::BuiltinTypeDescriptor {
                    ts_name: #ts_name.to_string(),
                };
                manager.registry(type_id, ::gents::Descriptor::BuiltinType(descriptor))
            }
            fn _ts_name() -> String {
                #ts_name.to_string()
            }
        }
    }
}
