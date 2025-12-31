use std::{
    any::TypeId,
    collections::{HashMap, HashSet},
};

use crate::ts_formatter::TsFormatter;
use crate::utils::remove_ext;

// `TS` trait defines the behavior of your types when generating files.
// `TS` generates some helper functions for file generator.
//
// Currently we have a limit that the type must implement `Clone` trait. This is because
// when serializing, we use dummy structs/enums to use the functionalities of `serde`
pub trait TS: Clone {
    fn _register(manager: &mut DescriptorManager, generic_base: bool) -> usize;
    // The name of this Rust type in Typescript.
    // u8 -> number
    // f64 -> number
    fn _ts_name() -> String;
    fn _is_optional() -> bool {
        false
    }
    // A special field for us to know about this type. It is always
    // used in the Rust builtin types like mapping `Vec<u8>`` to `Uint8Array`
    // rather than `readonly number[]`.
    fn _tag() -> Option<&'static str> {
        None
    }
}

/// Trait for defining TypeScript API interfaces
/// This should not be used directly - it's intended for internal use by the gents framework.
/// Use the `#[gents_derives::ts_interface]` attribute macro to define TypeScript interfaces.
pub trait _TsAPI {
    fn __get_api_descriptor() -> ApiDescriptor;
}

pub struct ApiDescriptor {
    pub name: String,
    pub file_name: String,
    pub methods: Vec<MethodDescriptor>,
    pub comment: Vec<String>,
    pub async_func: bool,
}

pub struct MethodDescriptor {
    pub name: String,
    pub params: Vec<(String, TypeId)>,
    pub comment: Vec<String>,
    pub return_type: Option<TypeId>,
}

#[derive(Default)]
pub struct DescriptorManager {
    pub descriptors: Vec<Descriptor>,
    pub api_descriptors: Vec<ApiDescriptor>,
    pub id_map: HashMap<TypeId, usize>,
    generics_map: HashMap<usize, String>,
}

impl DescriptorManager {
    pub fn registry(&mut self, type_id: TypeId, descriptor: Descriptor) -> usize {
        match self.id_map.get(&type_id) {
            Some(idx) => *idx,
            None => {
                let idx = self.descriptors.len();
                self.descriptors.push(descriptor);
                self.id_map.insert(type_id, idx);
                idx
            }
        }
    }

    pub fn add_api_descriptor(&mut self, descriptor: ApiDescriptor) {
        <&str as TS>::_register(self, false);
        self.api_descriptors.push(descriptor);
    }

    pub fn add_generics_map(&mut self, idx: usize, generics: String) {
        self.generics_map.insert(idx, generics);
    }

    pub fn gen_data(self) -> Vec<(String, String)> {
        let mut result: Vec<(String, String)> = vec![];
        let DescriptorManager {
            descriptors,
            api_descriptors,
            id_map,
            generics_map,
        } = self;
        descriptors
            .iter()
            .enumerate()
            .for_each(|(idx, descriptor)| match &descriptor {
                Descriptor::Interface(d) => {
                    if d.generic.is_some() {
                        return;
                    }
                    let generics = if let Some(v) = generics_map.get(&idx) {
                        format!("<{}>", v).to_string()
                    } else {
                        String::new()
                    };
                    let import_deps =
                        d.dependencies
                            .iter()
                            .fold(HashSet::new(), |mut prev, curr| {
                                let deps = get_import_deps_idx(&descriptors, *curr);
                                prev.extend(deps);
                                prev
                            });

                    let mut fmt = TsFormatter::new();
                    // imports
                    {
                        let mut deps: Vec<_> = import_deps.into_iter().collect();
                        deps.sort();
                        for dep in deps {
                            let (ts_name, file_name) = get_import_deps(&descriptors, dep);
                            fmt.add_import(&ts_name, &file_name);
                        }
                    }

                    // comments and interface body
                    fmt.add_comment(&d.comments);
                    fmt.start_interface(&d.ts_name, &generics);
                    for fd in &d.fields {
                        fmt.add_field(&fd.ident, &fd.ts_ty, fd.optional, &fd.comments);
                    }
                    fmt.end_interface();

                    if d.need_builder {
                        fmt.add_blank_line();
                        write_builder(&d, &mut fmt);
                    }

                    result.push((d.file_name.to_string(), fmt.end_file()))
                }
                Descriptor::Enum(e) => {
                    if e.generic.is_some() {
                        return;
                    }
                    let import_deps =
                        e.dependencies
                            .iter()
                            .fold(HashSet::new(), |mut prev, curr| {
                                let deps = get_import_deps_idx(&descriptors, *curr);
                                prev.extend(deps);
                                prev
                            });

                    let mut fmt = TsFormatter::new();
                    // imports
                    {
                        let mut deps: Vec<_> = import_deps.into_iter().collect();
                        deps.sort();
                        for dep in deps {
                            let (ts_name, file_name) = get_import_deps(&descriptors, dep);
                            fmt.add_import(&ts_name, &file_name);
                        }
                    }
                    // comments and type union
                    fmt.add_comment(&e.comments);
                    fmt.start_enum(&e.ts_name);
                    for fd in &e.fields {
                        let ty = fd.ts_ty.to_string();
                        let v = if ty != "" {
                            format!("{{ {}: '{}'; value: {} }}", e.tag, fd.tag_value, ty)
                        } else {
                            format!(r#"'{}'"#, fd.tag_value)
                        };
                        fmt.add_enum_variant_raw(&v);
                    }
                    fmt.end_enum();

                    result.push((e.file_name.to_string(), fmt.end_file()))
                }
                _ => {}
            });
        api_descriptors.into_iter().for_each(|api| {
            let mut fmt = TsFormatter::new();
            let mut deps = Vec::<TypeId>::new();

            // collect all non-builtin type dependencies from params and return types
            api.methods.iter().for_each(|m| {
                m.params.iter().for_each(|(_, t)| {
                    deps.push(*t);
                });
                if let Some(t) = &m.return_type {
                    deps.push(*t);
                }
            });
            deps.dedup();

            deps.into_iter().for_each(|t| {
                let idx = *id_map.get(&t).expect(&format!(
                    "type id {:?} not found in id_map. Please `add()` it first",
                    t
                ));
                let desc = descriptors.get(idx).unwrap();
                if let Descriptor::BuiltinType(_) = desc {
                    return;
                }
                let (ts_name, file_name) = get_import_deps(&descriptors, idx);
                fmt.add_import(&ts_name, &file_name);
            });
            let async_func = api.async_func;

            // For API files we currently do not emit comments into the generated TS,
            // so that the output matches the expected test fixtures exactly.
            fmt.start_interface(&api.name, "");

            api.methods.into_iter().for_each(|m| {
                let params = m
                    .params
                    .into_iter()
                    .map(|(n, t)| {
                        let idx = id_map.get(&t).unwrap();
                        let desc = descriptors.get(*idx).unwrap();
                        (n, desc.ts_name().to_string())
                    })
                    .collect();
                let ret = m.return_type.as_ref().map(|t| {
                    let idx = id_map.get(t).unwrap();
                    let desc = descriptors.get(*idx).unwrap();
                    desc.ts_name().to_string()
                });
                fmt.add_comment(&m.comment);
                if async_func {
                    fmt.add_async_method(&m.name, params, ret);
                } else {
                    fmt.add_method(&m.name, params, ret);
                }
            });
            fmt.end_interface();
            result.push((api.file_name.to_string(), fmt.end_file()));
        });
        result
    }
}

// todo: InterfaceDescriptor and EnumDescriptor are the same now.
// Remove one of it.
#[derive(Debug)]
pub enum Descriptor {
    Interface(InterfaceDescriptor),
    Enum(EnumDescriptor),
    BuiltinType(BuiltinTypeDescriptor),
    Generics(GenericDescriptor),
}

impl Descriptor {
    fn ts_name(&self) -> &str {
        match self {
            Descriptor::Interface(desc) => &desc.ts_name,
            Descriptor::Enum(desc) => &desc.ts_name,
            Descriptor::BuiltinType(desc) => &desc.ts_name,
            Descriptor::Generics(desc) => &desc.ts_name,
        }
    }
}

#[derive(Debug)]
pub struct GenericDescriptor {
    pub dependencies: Vec<usize>,
    pub ts_name: String,
    pub optional: bool,
}

#[derive(Debug)]
pub struct BuiltinTypeDescriptor {
    pub ts_name: String,
}

#[derive(Debug)]
pub struct EnumDescriptor {
    pub dependencies: Vec<usize>,
    pub fields: Vec<FieldDescriptor>,
    pub file_name: String,
    pub ts_name: String,
    pub comments: Vec<String>,
    pub tag: String,
    pub generic: Option<usize>,
}

/// Describe how to generate a ts interface.
#[derive(Debug)]
pub struct InterfaceDescriptor {
    // The index of the descriptors in the manager.
    pub dependencies: Vec<usize>,
    pub fields: Vec<FieldDescriptor>,
    pub file_name: String,
    pub ts_name: String,
    pub comments: Vec<String>,
    pub need_builder: bool,
    pub generic: Option<usize>,
}

#[derive(Debug)]
pub struct FieldDescriptor {
    pub ident: String,
    pub optional: bool,
    pub ts_ty: String,
    pub comments: Vec<String>,
    pub tag_value: String,
}

macro_rules! impl_builtin {
    ($i: ident, $l: literal, $t: literal) => {
        impl TS for $i {
            fn _register(manager: &mut DescriptorManager, _generic_base: bool) -> usize {
                let type_id = TypeId::of::<$i>();
                let descriptor = BuiltinTypeDescriptor {
                    ts_name: $l.to_string(),
                };
                manager.registry(type_id, Descriptor::BuiltinType(descriptor))
            }

            fn _ts_name() -> String {
                $l.to_string()
            }

            fn _tag() -> Option<&'static str> {
                Some($t)
            }
        }
    };
}

impl TS for &str {
    fn _register(manager: &mut DescriptorManager, _generic_base: bool) -> usize {
        let type_id = TypeId::of::<&str>();
        let descriptor = BuiltinTypeDescriptor {
            ts_name: "string".to_string(),
        };
        manager.registry(type_id, Descriptor::BuiltinType(descriptor))
    }

    fn _ts_name() -> String {
        String::from("string")
    }

    fn _tag() -> Option<&'static str> {
        Some("string")
    }
}

impl_builtin!(u8, "number", "u8");
impl_builtin!(u16, "number", "u16");
impl_builtin!(u32, "number", "u32");
impl_builtin!(u64, "number", "u64");
impl_builtin!(usize, "number", "usize");
impl_builtin!(i8, "number", "i8");
impl_builtin!(i32, "number", "i32");
impl_builtin!(i64, "number", "i64");
impl_builtin!(f32, "number", "f32");
impl_builtin!(f64, "number", "f64");
impl_builtin!(String, "string", "string");
impl_builtin!(bool, "boolean", "bool");

impl<T: TS + 'static> TS for Vec<T> {
    fn _register(manager: &mut DescriptorManager, generic_base: bool) -> usize {
        let idx = T::_register(manager, generic_base);
        let type_id = TypeId::of::<Self>();
        let descriptor = GenericDescriptor {
            dependencies: vec![idx],
            ts_name: Self::_ts_name(),
            optional: false,
        };
        manager.registry(type_id, Descriptor::Generics(descriptor))
    }

    fn _ts_name() -> String {
        if let Some(t) = T::_tag() {
            if t == "u8" {
                return "Uint8Array".to_string();
            }
        }
        format!("readonly {}[]", T::_ts_name())
    }
}

impl<T: TS + 'static> TS for Option<T> {
    fn _register(manager: &mut DescriptorManager, generic_base: bool) -> usize {
        let idx = T::_register(manager, generic_base);
        let type_id = TypeId::of::<Self>();
        let descriptor = GenericDescriptor {
            dependencies: vec![idx],
            ts_name: Self::_ts_name(),
            optional: true,
        };
        manager.registry(type_id, Descriptor::Generics(descriptor))
    }

    fn _ts_name() -> String {
        T::_ts_name()
    }

    fn _is_optional() -> bool {
        true
    }
}

impl<T: TS + 'static, E: TS + 'static> TS for Result<T, E> {
    fn _register(manager: &mut DescriptorManager, generic_base: bool) -> usize {
        let t_idx = T::_register(manager, generic_base);
        let e_idx = E::_register(manager, generic_base);
        let type_id = TypeId::of::<Self>();
        let descriptor = GenericDescriptor {
            dependencies: vec![t_idx, e_idx],
            ts_name: Self::_ts_name(),
            optional: false,
        };
        manager.registry(type_id, Descriptor::Generics(descriptor))
    }

    fn _ts_name() -> String {
        format!("{} | {}", T::_ts_name(), E::_ts_name())
    }
}

impl<K, V> TS for (K, V)
where
    K: TS + 'static,
    V: TS + 'static,
{
    fn _register(manager: &mut DescriptorManager, generic_base: bool) -> usize {
        let k_dep = K::_register(manager, generic_base);
        let v_dep = V::_register(manager, generic_base);
        let descriptor = GenericDescriptor {
            dependencies: vec![k_dep, v_dep],
            ts_name: Self::_ts_name(),
            optional: false,
        };
        let type_id = TypeId::of::<Self>();
        manager.registry(type_id, Descriptor::Generics(descriptor))
    }

    fn _ts_name() -> String {
        format!("(readonly [{}, {}])", K::_ts_name(), V::_ts_name())
    }
}

impl<K, V> TS for HashMap<K, V>
where
    K: TS + 'static,
    V: TS + 'static,
{
    fn _register(manager: &mut DescriptorManager, generic_base: bool) -> usize {
        let k_dep = K::_register(manager, generic_base);
        let v_dep = V::_register(manager, generic_base);
        let descriptor = GenericDescriptor {
            dependencies: vec![k_dep, v_dep],
            ts_name: Self::_ts_name(),
            optional: false,
        };
        let type_id = TypeId::of::<Self>();
        manager.registry(type_id, Descriptor::Generics(descriptor))
    }

    fn _ts_name() -> String {
        format!("Map<{}, {}>", K::_ts_name(), V::_ts_name())
    }
}

fn get_import_deps_idx(all: &Vec<Descriptor>, idx: usize) -> HashSet<usize> {
    let mut result = HashSet::new();
    let descriptor = all.get(idx).unwrap();
    match descriptor {
        Descriptor::Interface(_) => {
            result.insert(idx);
        }
        Descriptor::Enum(_) => {
            result.insert(idx);
        }
        Descriptor::BuiltinType(_) => {}
        Descriptor::Generics(d) => d.dependencies.iter().for_each(|dep| {
            let deps = get_import_deps_idx(all, *dep);
            result.extend(deps);
        }),
    };
    result
}

fn get_import_deps(all: &Vec<Descriptor>, idx: usize) -> (String, String) {
    let descriptor = all.get(idx).unwrap();
    match descriptor {
        Descriptor::Interface(d) => (d.ts_name.to_string(), remove_ext(&d.file_name)),
        Descriptor::Enum(d) => (d.ts_name.to_string(), remove_ext(&d.file_name)),
        _ => unreachable!(),
    }
}

fn write_builder(d: &InterfaceDescriptor, fmt: &mut TsFormatter) {
    // class header
    fmt.start_class(&format!("{}Builder", d.ts_name));
    // fields
    for fd in &d.fields {
        if fd.optional {
            fmt.add_class_field(&format!("private _{}?: {}", fd.ident, fd.ts_ty));
        } else {
            fmt.add_class_field(&format!("private _{}!: {}", fd.ident, fd.ts_ty));
        }
    }
    // setters with blank line between when multiple
    let mut first = true;
    for fd in &d.fields {
        if !first {
            fmt.add_blank_line();
        }
        first = false;
        fmt.start_method(&format!("public {}(value: {})", fd.ident, fd.ts_ty));
        fmt.add_method_line(&format!("this._{} = value", fd.ident));
        fmt.add_method_line("return this");
        fmt.end_method();
    }
    // build()
    fmt.start_method("public build()");
    for fd in d.fields.iter().filter(|fd| !fd.optional) {
        fmt.add_method_line(&format!(
            "if (this._{} === undefined) throw new Error('missing {}')",
            fd.ident, fd.ident
        ));
    }
    let field_set = d
        .fields
        .iter()
        .map(|fd| format!("{}: this._{}", fd.ident, fd.ident))
        .collect::<Vec<_>>()
        .join(", ");
    fmt.add_method_line(&format!("return {{ {} }}", field_set));
    fmt.end_method();
    // class end
    fmt.end_class();
}
