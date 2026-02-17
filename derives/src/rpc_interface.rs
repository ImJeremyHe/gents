use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input, DeriveInput, Error, Fields, Ident, LitStr, Result, Token, Type, TypeBareFn,
};

use crate::convert_camel_from_snake;

/// Parses the `#[ts(...)]` attributes for the Interface macro
struct InterfaceAttrs {
    file_name: String,
    rename_all: Option<RenameAll>,
}

#[derive(Clone, Copy)]
enum RenameAll {
    CamelCase,
}

impl Parse for InterfaceAttrs {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut file_name: Option<String> = None;
        let mut rename_all: Option<RenameAll> = None;

        while !input.is_empty() {
            let ident: Ident = input.parse()?;
            input.parse::<Token![=]>()?;
            let lit: LitStr = input.parse()?;

            if ident == "file_name" {
                file_name = Some(lit.value());
            } else if ident == "rename_all" {
                match lit.value().as_str() {
                    "camelCase" => rename_all = Some(RenameAll::CamelCase),
                    _ => return Err(Error::new_spanned(lit, "expected `camelCase`")),
                }
            } else {
                return Err(Error::new_spanned(
                    ident,
                    "expected `file_name = \"...\"` or `rename_all = \"camelCase\"`",
                ));
            }

            if input.is_empty() {
                break;
            }
            input.parse::<Token![,]>()?;
        }

        let file_name =
            file_name.ok_or_else(|| input.error("`file_name = \"...\"` is required"))?;

        Ok(Self {
            file_name,
            rename_all,
        })
    }
}

/// Represents a parsed parameter
struct RpcParam {
    name: String,
    ty: Type,
}

/// Represents a parsed RPC method field
struct RpcMethod {
    name: String,
    params: Vec<RpcParam>,
    ok_type: Type,
    err_type: Type,
}

pub fn derive_interface(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match expand_interface(&input) {
        Ok(ts) => ts.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

fn expand_interface(input: &DeriveInput) -> Result<proc_macro2::TokenStream> {
    // Parse `#[ts(...)]` attribute
    let attrs = parse_ts_attrs(input)?;
    let file_name = attrs.file_name;
    let rename_all = attrs.rename_all;
    let ident = &input.ident;
    let ts_name = ident.to_string();

    // Must be a struct
    let fields = match &input.data {
        syn::Data::Struct(ds) => &ds.fields,
        _ => {
            return Err(Error::new_spanned(
                ident,
                "Interface can only be derived for structs",
            ))
        }
    };

    // Must be named fields
    let named_fields = match fields {
        Fields::Named(f) => &f.named,
        _ => {
            return Err(Error::new_spanned(
                ident,
                "Interface struct must have named fields",
            ))
        }
    };

    // Parse each field as an RPC method
    let mut methods = Vec::new();
    for field in named_fields {
        let field_ident = field
            .ident
            .as_ref()
            .ok_or_else(|| Error::new_spanned(&field.ty, "expected named field"))?;

        let method = parse_fn_field(field_ident, &field.ty, rename_all)?;
        methods.push(method);
    }

    // Generate the method descriptors
    let method_tokens: Vec<_> = methods
        .iter()
        .map(|m| {
            let name = &m.name;
            let ok_ty = &m.ok_type;
            let err_ty = &m.err_type;

            let param_tokens: Vec<_> = m
                .params
                .iter()
                .map(|p| {
                    let param_name = &p.name;
                    let param_ty = &p.ty;
                    quote! {
                        (#param_name.to_string(), std::any::TypeId::of::<#param_ty>())
                    }
                })
                .collect();

            quote! {
                ::gents::RpcMethodDescriptor {
                    name: #name.to_string(),
                    params: vec![ #(#param_tokens),* ],
                    ok_type: std::any::TypeId::of::<#ok_ty>(),
                    err_type: std::any::TypeId::of::<#err_ty>(),
                }
            }
        })
        .collect();

    // Generate type registrations
    let register_tokens: Vec<_> = methods
        .iter()
        .flat_map(|m| {
            let ok_ty = &m.ok_type;
            let err_ty = &m.err_type;
            let mut tokens = vec![
                quote! { <#ok_ty as ::gents::TS>::_register(manager, true); },
                quote! { <#err_ty as ::gents::TS>::_register(manager, true); },
            ];
            for p in &m.params {
                let param_ty = &p.ty;
                tokens.push(quote! { <#param_ty as ::gents::TS>::_register(manager, true); });
            }
            tokens
        })
        .collect();

    let expanded = quote! {
        impl ::gents::_TsRpcInterface for #ident {
            fn __get_rpc_descriptor(manager: &mut ::gents::DescriptorManager) -> ::gents::RpcInterfaceDescriptor {
                #(#register_tokens)*
                ::gents::RpcInterfaceDescriptor {
                    name: #ts_name.to_string(),
                    file_name: #file_name.to_string(),
                    methods: vec![ #(#method_tokens),* ],
                }
            }
        }
    };

    Ok(expanded)
}

fn parse_ts_attrs(input: &DeriveInput) -> Result<InterfaceAttrs> {
    for attr in &input.attrs {
        if attr.path().is_ident("ts") {
            return attr.parse_args::<InterfaceAttrs>();
        }
    }
    Err(Error::new(
        Span::call_site(),
        "missing #[ts(file_name = \"...\")] attribute",
    ))
}

fn parse_fn_field(
    field_ident: &Ident,
    ty: &Type,
    rename_all: Option<RenameAll>,
) -> Result<RpcMethod> {
    // Convert method name based on rename_all
    let name = match rename_all {
        Some(RenameAll::CamelCase) => convert_camel_from_snake(field_ident.to_string()),
        None => field_ident.to_string(),
    };

    // Extract the bare function type
    let bare_fn = match ty {
        Type::BareFn(f) => f,
        _ => {
            return Err(Error::new_spanned(
                ty,
                "Interface fields must be function types: fn(name: Type, ...) -> Result<T, E>",
            ))
        }
    };

    // Parse parameters
    let params = parse_params(bare_fn, rename_all)?;

    // Parse return type (must be Result<T, E>)
    let (ok_type, err_type) = parse_result_return_type(bare_fn)?;

    Ok(RpcMethod {
        name,
        params,
        ok_type,
        err_type,
    })
}

fn parse_params(bare_fn: &TypeBareFn, rename_all: Option<RenameAll>) -> Result<Vec<RpcParam>> {
    let mut params = Vec::new();

    for (idx, arg) in bare_fn.inputs.iter().enumerate() {
        // Get parameter name, convert based on rename_all
        let param_name = match &arg.name {
            Some((ident, _)) => match rename_all {
                Some(RenameAll::CamelCase) => convert_camel_from_snake(ident.to_string()),
                None => ident.to_string(),
            },
            None => format!("arg{}", idx), // fallback if no name
        };

        params.push(RpcParam {
            name: param_name,
            ty: arg.ty.clone(),
        });
    }

    Ok(params)
}

fn parse_result_return_type(bare_fn: &TypeBareFn) -> Result<(Type, Type)> {
    let ret_type = match &bare_fn.output {
        syn::ReturnType::Type(_, ty) => ty.as_ref(),
        syn::ReturnType::Default => {
            return Err(Error::new_spanned(
                bare_fn,
                "RPC method must have a return type: Result<T, E>",
            ))
        }
    };

    // Extract Result<T, E>
    let type_path = match ret_type {
        Type::Path(p) => p,
        _ => {
            return Err(Error::new_spanned(
                ret_type,
                "return type must be Result<T, E>",
            ))
        }
    };

    let last_segment = type_path
        .path
        .segments
        .last()
        .ok_or_else(|| Error::new_spanned(ret_type, "invalid return type"))?;

    if last_segment.ident != "Result" {
        return Err(Error::new_spanned(
            ret_type,
            "return type must be Result<T, E>",
        ));
    }

    let args = match &last_segment.arguments {
        syn::PathArguments::AngleBracketed(args) => args,
        _ => {
            return Err(Error::new_spanned(
                ret_type,
                "Result must have type parameters: Result<T, E>",
            ))
        }
    };

    if args.args.len() != 2 {
        return Err(Error::new_spanned(
            ret_type,
            "Result must have exactly two type parameters: Result<T, E>",
        ));
    }

    let ok_type = match &args.args[0] {
        syn::GenericArgument::Type(t) => t.clone(),
        _ => return Err(Error::new_spanned(&args.args[0], "expected type argument")),
    };

    let err_type = match &args.args[1] {
        syn::GenericArgument::Type(t) => t.clone(),
        _ => return Err(Error::new_spanned(&args.args[1], "expected type argument")),
    };

    Ok((ok_type, err_type))
}
