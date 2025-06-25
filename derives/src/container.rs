use proc_macro2::Ident;
use syn::parse::Parse;
use syn::punctuated::Punctuated;
use syn::token::Comma;
use syn::Attribute;
use syn::MetaNameValue;
use syn::Type;

use crate::symbol::BUILDER;
use crate::symbol::FILE_NAME;
use crate::symbol::TAG;
use crate::symbol::TAG_VALUE;
use crate::symbol::{RENAME, RENAME_ALL, SKIP, TS};

pub struct Container<'a> {
    pub file_name: String,
    pub is_enum: bool,
    pub fields: Vec<Field<'a>>,
    pub rename_all: Option<RenameAll>,
    pub rename: Option<String>,
    pub ident: &'a Ident,
    pub comments: Vec<String>,
    pub need_builder: bool,
    pub tag: Option<String>,
}

impl<'a> Container<'a> {
    pub fn from_ast(item: &'a syn::DeriveInput) -> Self {
        let mut rename_all: Option<RenameAll> = None;
        let mut file_name: Option<String> = None;
        let mut rename: Option<String> = None;
        let mut need_builder = false;
        let mut tag: Option<String> = None;
        let comments = parse_comments(&item.attrs);
        for meta_item in item
            .attrs
            .iter()
            .flat_map(|attr| get_ts_meta_name_value_items(attr))
            .flatten()
        {
            let m = meta_item;
            if m.path == RENAME_ALL {
                let s = get_lit_str(&m.value).expect("rename_all requires lit str");
                let t = match s.value().as_str() {
                    "camelCase" => RenameAll::CamelCase,
                    _ => panic!("unexpected literal for case converting"),
                };
                rename_all = Some(t);
            } else if m.path == FILE_NAME {
                let s = get_lit_str(&m.value).expect("file_name requires lit str");
                file_name = Some(s.value());
            } else if m.path == RENAME {
                let s = get_lit_str(&m.value).expect("rename requires lit str");
                rename = Some(s.value());
            } else if m.path == TAG {
                let s = get_lit_str(&m.value).expect("tag requires lit str");
                tag = Some(s.value());
            } else {
                panic!("unexpected attr")
            }
        }
        for path in item
            .attrs
            .iter()
            .flat_map(|attr| get_ts_meta_path_items(attr))
            .flatten()
        {
            if path == BUILDER {
                need_builder = true;
            }
        }
        match &item.data {
            syn::Data::Struct(ds) => {
                if tag.is_some() {
                    panic!("struct types doesn't support tag")
                }
                let fields = ds
                    .fields
                    .iter()
                    .map(|f| Field::from_field(f))
                    .collect::<Vec<_>>();
                Container {
                    file_name: file_name.expect("file name is required"),
                    is_enum: false,
                    fields,
                    rename_all,
                    ident: &item.ident,
                    rename,
                    comments,
                    need_builder,
                    tag,
                }
            }
            syn::Data::Enum(e) => {
                if need_builder {
                    panic!("enum does not support builder");
                }
                let fields = e
                    .variants
                    .iter()
                    .map(|v| Field::from_variant(v))
                    .collect::<Vec<_>>();
                Container {
                    file_name: file_name.unwrap(),
                    is_enum: true,
                    fields,
                    rename_all,
                    ident: &item.ident,
                    rename,
                    comments,
                    need_builder,
                    tag,
                }
            }
            _ => panic!("gents does not support the union type currently, use struct instead"),
        }
    }
}

pub struct Field<'a> {
    pub rename: Option<String>,
    pub ident: &'a Ident,
    pub ty: Option<&'a Type>, // enum ty can be None.
    pub skip: bool,
    pub comments: Vec<String>,
    pub tag_value: Option<String>,
}

impl<'a> Field<'a> {
    pub fn from_field(f: &'a syn::Field) -> Self {
        let comments = parse_comments(&f.attrs);
        let attrs = parse_attrs(&f.attrs);
        Field {
            rename: attrs.rename,
            ident: f.ident.as_ref().unwrap(),
            ty: Some(&f.ty),
            skip: attrs.skip,
            comments,
            tag_value: attrs.tag_value,
        }
    }

    pub fn from_variant(v: &'a syn::Variant) -> Self {
        let comments = parse_comments(&v.attrs);
        let attrs = parse_attrs(&v.attrs);
        if v.fields.len() > 1 {
            panic!("not implemented yet")
        }
        let field = &v.fields.iter().next();
        let ty = match field {
            Some(f) => Some(&f.ty),
            None => None,
        };
        Field {
            rename: attrs.rename,
            ident: &v.ident,
            ty,
            skip: attrs.skip,
            comments,
            tag_value: attrs.tag_value,
        }
    }
}

fn parse_attrs<'a>(attrs: &'a Vec<Attribute>) -> FieldAttrs {
    let mut skip = false;
    let mut rename: Option<String> = None;
    let mut tag_value: Option<String> = None;
    for meta_item in attrs
        .iter()
        .flat_map(|attr| get_ts_meta_name_value_items(attr))
        .flatten()
    {
        let m = meta_item;
        if m.path == RENAME {
            if let Ok(s) = get_lit_str(&m.value) {
                rename = Some(s.value());
            }
        } else if m.path == SKIP {
            if let Ok(s) = get_lit_bool(&m.value) {
                skip = s;
            } else {
                panic!("expected bool value in skip attr")
            }
        } else if m.path == TAG_VALUE {
            if let Ok(s) = get_lit_str(&m.value) {
                tag_value = Some(s.value());
            }
        }
    }
    FieldAttrs {
        skip,
        rename,
        tag_value,
    }
}

struct FieldAttrs {
    skip: bool,
    rename: Option<String>,
    tag_value: Option<String>,
}

pub enum RenameAll {
    CamelCase,
}

fn get_ts_meta_name_value_items(attr: &syn::Attribute) -> Result<Vec<syn::MetaNameValue>, ()> {
    if attr.path() != TS {
        return Ok(Vec::new());
    }

    match attr.parse_args_with(Punctuated::<MetaNameValue, Comma>::parse_terminated) {
        Ok(name_values) => Ok(name_values.into_iter().collect()),
        Err(_) => Err(()),
    }
}

fn get_ts_meta_path_items(attr: &syn::Attribute) -> Result<Vec<syn::Path>, ()> {
    if attr.path() != TS {
        return Ok(Vec::new());
    }

    match attr.parse_args_with(Punctuated::<syn::Path, Comma>::parse_terminated) {
        Ok(name_values) => Ok(name_values.into_iter().collect()),
        Err(_) => Err(()),
    }
}

fn get_lit_str<'a>(lit: &'a syn::Expr) -> Result<&'a syn::LitStr, ()> {
    if let syn::Expr::Lit(lit) = lit {
        if let syn::Lit::Str(l) = &lit.lit {
            return Ok(&l);
        }
    }
    Err(())
}

fn get_lit_bool<'a>(lit: &'a syn::Expr) -> Result<bool, ()> {
    if let syn::Expr::Lit(lit) = lit {
        if let syn::Lit::Bool(b) = &lit.lit {
            return Ok(b.value);
        }
    }
    Err(())
}

fn parse_comments(attrs: &[Attribute]) -> Vec<String> {
    let mut result = Vec::new();

    attrs.iter().for_each(|attr| {
        if attr.path().is_ident("doc") {
            if let Ok(nv) = attr.meta.require_name_value() {
                if let Ok(s) = get_lit_str(&nv.value) {
                    let comment = s.value();
                    result.push(comment.trim().to_string());
                }
            }
        }
    });
    result
}

pub(crate) struct GentsWasmAttrs {
    file_name: String,
}

impl GentsWasmAttrs {
    pub fn get_file_name(&self) -> &str {
        &self.file_name
    }
}

impl Parse for GentsWasmAttrs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let name_values = Punctuated::<MetaNameValue, Comma>::parse_terminated(input)?;
        let mut file_name = String::new();
        name_values.into_iter().for_each(|name_value| {
            let path = name_value.path;
            let attr = path
                .get_ident()
                .expect("unvalid attr, should be an ident")
                .to_string();
            let value = get_lit_str(&name_value.value)
                .expect("should be a str")
                .value();
            match attr.as_str() {
                "file_name" => file_name = value,
                _ => panic!("invalid attr: {}", attr),
            }
        });
        if file_name.is_empty() {
            panic!("file_name unset")
        }
        Ok(GentsWasmAttrs { file_name })
    }
}
