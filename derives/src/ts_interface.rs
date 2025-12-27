use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input, Error, Ident, ImplItem, ImplItemFn, ItemImpl, LitStr, Result, Token, Type,
};

use crate::convert_camel_from_snake;

/// #[ts_interface(file_name = "a.ts")]
pub fn ts_interface(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr as TsInterfaceArgs);
    let impl_block = parse_macro_input!(item as ItemImpl);

    match expand_ts_interface(args, impl_block) {
        Ok(ts) => ts.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

struct TsInterfaceArgs {
    file_name: String,
    ident: Option<String>,
}

impl Parse for TsInterfaceArgs {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut file_name: Option<String> = None;
        let mut ident_val: Option<String> = None;

        while !input.is_empty() {
            let ident: Ident = input.parse()?;
            input.parse::<Token![=]>()?;
            let lit: LitStr = input.parse()?;

            if ident == "file_name" {
                file_name = Some(lit.value());
            } else if ident == "ident" {
                ident_val = Some(lit.value());
            } else {
                return Err(Error::new_spanned(
                    ident,
                    "expected `file_name = \"...\"` or `ident = \"...\"`",
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
            ident: ident_val,
        })
    }
}

fn expand_ts_interface(
    args: TsInterfaceArgs,
    impl_block: ItemImpl,
) -> Result<proc_macro2::TokenStream> {
    // Reject trait impl
    if impl_block.trait_.is_some() {
        return Err(Error::new_spanned(
            impl_block.impl_token,
            "#[ts_interface] only supports inherent impl blocks",
        ));
    }

    // Extract self type ident
    let self_ty_ident = extract_self_type_ident(&impl_block.self_ty)?;

    // prefer user-specified ident if provided, otherwise fall back to Rust type name
    let type_name = args.ident.unwrap_or_else(|| self_ty_ident.to_string());
    let file_name = args.file_name;

    // Collect impl-level doc comments
    let mut impl_comments = Vec::new();
    for attr in &impl_block.attrs {
        if attr.path().is_ident("doc") {
            if let Ok(lit) = attr.parse_args::<LitStr>() {
                impl_comments.push(lit.value());
            }
        }
    }

    // Collect public methods
    let mut method_tokens = Vec::new();

    for item in &impl_block.items {
        let ImplItem::Fn(func) = item else { continue };

        // Only `pub fn`
        if !matches!(func.vis, syn::Visibility::Public(_)) {
            continue;
        }

        method_tokens.push(expand_method(func)?);
    }

    let expanded = quote! {
        #impl_block

        impl gents::_TsAPI for #self_ty_ident {
            fn __get_api_descriptor() -> gents::ApiDescriptor {
                gents::ApiDescriptor {
                    name: #type_name.to_string(),
                    file_name: #file_name.to_string(),
                    methods: vec![ #(#method_tokens),* ],
                    comment: vec![ #( #impl_comments.to_string() ),* ],
                }
            }
        }
    };

    Ok(expanded)
}

fn extract_self_type_ident(ty: &Type) -> Result<&Ident> {
    match ty {
        Type::Path(p) => p
            .path
            .segments
            .last()
            .map(|s| &s.ident)
            .ok_or_else(|| Error::new_spanned(ty, "invalid self type")),
        _ => Err(Error::new_spanned(
            ty,
            "unsupported self type for #[ts_interface]",
        )),
    }
}

fn expand_method(func: &ImplItemFn) -> Result<proc_macro2::TokenStream> {
    let name = convert_camel_from_snake(func.sig.ident.to_string());

    // Collect method-level doc comments
    let mut comments = Vec::new();
    for attr in &func.attrs {
        if attr.path().is_ident("doc") {
            if let Ok(lit) = attr.parse_args::<LitStr>() {
                comments.push(lit.value());
            }
        }
    }

    // Parameters
    let mut params = Vec::new();

    for arg in &func.sig.inputs {
        match arg {
            syn::FnArg::Receiver(_) => {
                // &self / &mut self — ignored
            }
            syn::FnArg::Typed(pat) => {
                let ident = match &*pat.pat {
                    syn::Pat::Ident(i) => convert_camel_from_snake(i.ident.to_string()),
                    _ => {
                        return Err(Error::new_spanned(
                            &pat.pat,
                            "unsupported parameter pattern",
                        ))
                    }
                };

                let ty = strip_reference(&*pat.ty);

                params.push(quote! {
                    (#ident.to_string(), std::any::TypeId::of::<#ty>())
                });
            }
        }
    }

    // Return type (required)
    let ret_ty = match &func.sig.output {
        syn::ReturnType::Type(_, ty) => {
            let ty = strip_reference(&*ty);
            Some(ty)
        }
        syn::ReturnType::Default => None,
    };

    let res = if let Some(ty) = ret_ty {
        quote! {
            gents::MethodDescriptor {
                name: #name.to_string(),
                params: vec![ #(#params),* ],
                comment: vec![ #( #comments.to_string() ),* ],
                return_type: Some(std::any::TypeId::of::<#ty>()),
            }
        }
    } else {
        quote! {
            gents::MethodDescriptor {
                name: #name.to_string(),
                params: vec![ #(#params),* ],
                comment: vec![ #( #comments.to_string() ),* ],
                return_type: None,
            }
        }
    };

    Ok(res)
}

fn strip_reference(ty: &syn::Type) -> &syn::Type {
    match ty {
        syn::Type::Reference(r) => {
            if let syn::Type::Path(p) = &*r.elem {
                if let Some(seg) = p.path.segments.last() {
                    if seg.ident == "str" {
                        return ty;
                    }
                }
            }
            &*r.elem
        }
        _ => ty,
    }
}
