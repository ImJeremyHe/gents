use crate::container::Container;
use quote::{format_ident, quote};
use syn::{parse::Parser, DeriveInput};

pub fn get_serde_impl_block(
    container: Container,
    derive_input: &DeriveInput,
) -> proc_macro2::TokenStream {
    if container.is_enum {
        get_serde_enum_impl_block(container, derive_input)
    } else {
        get_serde_struct_impl_block(container, derive_input)
    }
}

fn get_serde_enum_impl_block(
    container: Container,
    _derive_input: &DeriveInput,
) -> proc_macro2::TokenStream {
    if !container.is_enum {
        panic!("not support struct");
    }
    let generics = container.generics;
    let mut unit_variants = Vec::new();
    let mut non_unit_variants = Vec::new();
    container.fields.into_iter().for_each(|f| {
        if f.ty.is_none() {
            unit_variants.push(f);
        } else {
            non_unit_variants.push(f);
        }
    });
    let unit_ident = format_ident!("_GentsDummyUnitEnum{}", container.ident);
    let unit_variant_dummy_enum = if !unit_variants.is_empty() {
        let fields = unit_variants
            .iter()
            .map(|f| {
                let ident = f.ident;
                let rename = if f.rename.is_some() {
                    let rename = f.rename.as_ref().unwrap();
                    quote! {
                        #[serde(rename = #rename)]
                    }
                } else {
                    quote! {}
                };
                let skip = if f.skip {
                    quote! {
                        #[serde(skip)]
                    }
                } else {
                    quote! {}
                };
                quote! {
                    #rename
                    #skip
                    #ident,
                }
            })
            .collect::<Vec<_>>();
        quote! {
            #[derive(::gents::serde::Serialize, ::gents::serde::Deserialize)]
            #[serde(rename_all = "camelCase")]
            #[serde(untagged)]
            enum #unit_ident {
                #(#fields)*
            }
        }
    } else {
        quote! {}
    };
    let non_unit_ident = format_ident!("_GentsDummyNonUnitEnum{}", container.ident);
    let non_unit_variant_dummy_enum = if !non_unit_variants.is_empty() {
        let fields = non_unit_variants
            .iter()
            .map(|f| {
                let ident = f.ident;
                let rename = if f.rename.is_some() {
                    let rename = f.rename.as_ref().unwrap();
                    quote! {
                        #[serde(rename = #rename)]
                    }
                } else {
                    quote! {}
                };
                let skip = if f.skip {
                    quote! {
                        #[serde(skip)]
                    }
                } else {
                    quote! {}
                };
                let ty = f.ty.as_ref().unwrap();
                quote! {
                    #rename
                    #skip
                    #ident(#ty),
                }
            })
            .collect::<Vec<_>>();
        let tag = container.tag.expect("tag is required");
        quote! {
            #[derive(::gents::serde::Serialize, ::gents::serde::Deserialize)]
            #[serde(rename_all = "camelCase")]
            #[serde(tag = #tag, content="value")]
            enum #non_unit_ident<#(#generics),*> {
                #(#fields)*
            }
        }
    } else {
        quote! {}
    };

    let dummy_ident = format_ident!("_GentsDummy{}", container.ident);
    let dummy_unit_variant = if unit_variants.is_empty() {
        quote! {}
    } else {
        quote! {
            UnitDummy(#unit_ident),
        }
    };
    let dummy_non_unit_variant = if non_unit_variants.is_empty() {
        quote! {}
    } else {
        quote! {
            TaggedDummy(#non_unit_ident<#(#generics),*>),
        }
    };
    let dummy_enum = quote! {
        #unit_variant_dummy_enum
        #non_unit_variant_dummy_enum

        #[derive(::gents::serde::Serialize, ::gents::serde::Deserialize)]
        #[serde(rename_all = "camelCase")]
        #[serde(untagged)]
        enum #dummy_ident<#(#generics),*> {
            #dummy_unit_variant
            #dummy_non_unit_variant
        }

    };
    let generic_ser_bound = generics
        .iter()
        .map(|g| quote! { #g: ::serde::Serialize + ::gents::TS + Clone})
        .collect::<Vec<_>>();
    let generic_de_bound = generics
        .iter()
        .map(|g| quote! { #g: ::serde::Deserialize<'de> + ::gents::TS + Clone })
        .collect::<Vec<_>>();
    let ident = container.ident;
    let serde_impl = {
        let unit_ser = unit_variants
            .iter()
            .map(|v| {
                let ident = v.ident;
                quote! {Self::#ident => #dummy_ident::UnitDummy(#unit_ident::#ident),}
            })
            .collect::<Vec<_>>();
        let tagged_ser = non_unit_variants
            .iter()
            .map(|v| {
                let ident = v.ident;
                quote! {
                    Self::#ident(value) => {
                        #dummy_ident::TaggedDummy(#non_unit_ident::#ident(value.clone()))
                    },
                }
            })
            .collect::<Vec<_>>();
        let unit_de = unit_variants
            .iter()
            .map(|v| {
                let ident = v.ident;
                quote! {#dummy_ident::UnitDummy(#unit_ident::#ident) => Self::#ident,}
            })
            .collect::<Vec<_>>();
        let tagged_de = non_unit_variants
            .iter()
            .map(|v| {
                let ident = v.ident;
                quote! {#dummy_ident::TaggedDummy(#non_unit_ident::#ident(value)) => Self::#ident(value),}
            })
            .collect::<Vec<_>>();
        quote! {
            impl<#(#generic_ser_bound),*> ::gents::serde::Serialize for #ident<#(#generics),*> {
                fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                    where S: ::gents::serde::Serializer
                {
                    let dummy = match self {
                        #(#unit_ser)*
                        #(#tagged_ser)*
                    };
                    dummy.serialize(serializer)
                }
            }

            impl<'de, #(#generic_de_bound),*> ::gents::serde::Deserialize<'de> for #ident<#(#generics),*> {
                fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                    where D: ::gents::serde::Deserializer<'de>
                {
                    let dummy = #dummy_ident::deserialize(deserializer)?;
                    Ok(match dummy {
                        #(#unit_de)*
                        #(#tagged_de)*
                    })
                }
            }
        }
    };
    quote! {
        #dummy_enum
        #serde_impl
    }
}

// In this function, we will create a dummy struct/enum to implement serde traits.
//
// In this way, we can reuse the `serde` implementation of the struct/enum.
fn get_serde_struct_impl_block(
    container: Container,
    derive_input: &DeriveInput,
) -> proc_macro2::TokenStream {
    if container.is_enum {
        panic!("not support enum");
    }
    let mut dummy = derive_input.clone();
    dummy.attrs.clear();
    dummy.vis = syn::Visibility::Inherited;
    dummy.ident = format_ident!("_GentsDummy{}", dummy.ident);
    match &mut dummy.data {
        syn::Data::Struct(d) => {
            d.fields.iter_mut().enumerate().for_each(|(i, f)| {
                f.attrs.clear();
                if container.fields[i].skip {
                    let attr = quote! {#[serde(skip)]};
                    f.attrs
                        .extend(syn::Attribute::parse_outer.parse2(attr).unwrap());
                }
                if container.fields[i].rename.is_some() {
                    let rename = container.fields[i].rename.as_ref().unwrap();
                    let attr = quote! {#[serde(rename = #rename)]};
                    f.attrs
                        .extend(syn::Attribute::parse_outer.parse2(attr).unwrap());
                }
            });
        }
        _ => panic!("not support"),
    }

    let generic_ser_bound = container
        .generics
        .iter()
        .map(|g| quote! { #g: ::serde::Serialize + ::gents::TS + Clone})
        .collect::<Vec<_>>();
    let generic_de_bound = container
        .generics
        .iter()
        .map(|g| quote! { #g: ::serde::Deserialize<'de> + ::gents::TS + Clone })
        .collect::<Vec<_>>();
    let generic_ts_bound = container
        .generics
        .iter()
        .map(|g| quote! { #g: ::gents::TS })
        .collect::<Vec<_>>();
    let generic = container.generics;
    let dummy_ident = &dummy.ident;
    let ident = container.ident;

    let from = {
        let fields = container
            .fields
            .iter()
            .map(|f| {
                let ident = f.ident;
                quote! {
                    #ident: value.#ident,
                }
            })
            .collect::<Vec<_>>();

        quote! {
            impl<#(#generic_ts_bound),*> From<#dummy_ident<#(#generic),*>> for #ident<#(#generic),*> {
                fn from(value: #dummy_ident<#(#generic),*>) -> Self {
                    Self {
                        #(#fields)*
                    }
                }
            }
        }
    };

    let dummy_type = quote! {
        #[derive(::gents::serde::Serialize, ::gents::serde::Deserialize)]
        #[serde(rename_all = "camelCase")]
        #dummy
    };

    let serde = {
        quote! {
            impl<#(#generic_ser_bound),*> ::gents::serde::Serialize for #ident<#(#generic),*> {
                fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                    where S: ::gents::serde::Serializer
                {
                    let dummy: &#dummy_ident<#(#generic),*> = unsafe { std::mem::transmute(self) };
                    #dummy_ident::serialize(dummy, serializer)
                }
            }

            impl<'de, #(#generic_de_bound),*> ::gents::serde::Deserialize<'de> for #ident<#(#generic),*> {
                fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                    where D: ::gents::serde::Deserializer<'de>
                {
                    let dummy = #dummy_ident::<#(#generic),*>::deserialize(deserializer)?;
                    Ok(dummy.into())
                }
            }
        }
    };
    quote! {
        #dummy_type
        #from
        #serde
    }
}
