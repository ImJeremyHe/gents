use std::collections::BTreeSet;

#[derive(Default)]
pub struct TsFormatter {
    imports: BTreeSet<String>,
    lines: Vec<String>,
    indent: usize,
    // enum building state
    enum_mode: bool,
    enum_variants: Vec<String>,
}

impl TsFormatter {
    pub fn new() -> Self {
        Self {
            imports: BTreeSet::new(),
            lines: vec![],
            indent: 0,
            enum_mode: false,
            enum_variants: vec![],
        }
    }

    pub fn add_import(&mut self, ts_name: &str, file_name_no_ext: &str) {
        self.imports.insert(format!(
            "import {{ {} }} from './{}'",
            ts_name, file_name_no_ext
        ));
    }

    pub fn add_blank_line(&mut self) {
        self.lines.push(String::new());
    }

    pub fn add_comment(&mut self, comments: &[String]) {
        if comments.is_empty() {
            return;
        }
        for (i, line) in comments.iter().enumerate() {
            if i == 0 {
                self.write_line(&format!("// {}", line));
            } else {
                self.write_line(&format!("// {}", line));
            }
        }
    }

    pub fn start_interface(&mut self, name: &str, generics: &str) {
        self.write_line(&format!("export interface {}{} {{", name, generics));
        self.indent += 1;
    }

    pub fn add_field(&mut self, name: &str, ty: &str, optional: bool, comments: &[String]) {
        if !comments.is_empty() {
            for c in comments {
                self.write_line(&format!("// {}", c));
            }
        }
        if optional {
            self.write_line(&format!("{}?: {}", name, ty));
        } else {
            self.write_line(&format!("{}: {}", name, ty));
        }
    }

    pub fn add_method(&mut self, name: &str, params: Vec<(String, String)>, ret: Option<String>) {
        let param_str = params
            .iter()
            .map(|(n, t)| format!("{}: {}", n, t))
            .collect::<Vec<_>>()
            .join(", ");
        let ret_str = ret.map_or("void".to_string(), |r| r);
        self.write_line(&format!("{}({}): {};", name, param_str, ret_str));
    }

    pub fn end_interface(&mut self) {
        if self.indent > 0 {
            self.indent -= 1;
        }
        self.write_line("}");
    }

    // Class and methods helpers
    pub fn start_class(&mut self, name: &str) {
        self.write_line(&format!("export class {} {{", name));
        self.indent += 1;
    }

    pub fn add_class_field(&mut self, decl: &str) {
        // decl like: "private _f1!: number" (no semicolon)
        self.write_line(decl);
    }

    pub fn start_method(&mut self, signature: &str) {
        // signature like: "public f1(value: number)"
        self.write_line(&format!("{} {{", signature));
        self.indent += 1;
    }

    pub fn add_method_line(&mut self, line: &str) {
        self.write_line(line);
    }

    pub fn end_method(&mut self) {
        if self.indent > 0 {
            self.indent -= 1;
        }
        self.write_line("}");
    }

    pub fn end_class(&mut self) {
        if self.indent > 0 {
            self.indent -= 1;
        }
        self.write_line("}");
    }

    pub fn start_enum(&mut self, name: &str) {
        self.enum_mode = true;
        self.enum_variants.clear();
        self.write_line(&format!("export type {} =", name));
    }

    pub fn add_enum_variant_raw(&mut self, raw: &str) {
        if self.enum_mode {
            self.enum_variants.push(raw.to_string());
        }
    }

    pub fn end_enum(&mut self) {
        if self.enum_mode {
            let indent = self.current_indent_string(1);
            for v in self.enum_variants.iter() {
                self.lines.push(format!("{}| {}", indent, v));
            }
            self.enum_mode = false;
            self.enum_variants.clear();
        }
    }

    pub fn end_file(self) -> String {
        let mut out = String::new();
        if !self.imports.is_empty() {
            for i in &self.imports {
                out.push_str(i);
                out.push('\n');
            }
            out.push('\n');
        }
        for l in self.lines {
            if l.is_empty() {
                out.push('\n');
            } else {
                out.push_str(&l);
                // no semicolons; rely on ASI style
                out.push('\n');
            }
        }
        out
    }

    fn write_line(&mut self, content: &str) {
        let indent = self.current_indent_string(0);
        self.lines.push(format!("{}{}", indent, content));
    }

    fn current_indent_string(&self, extra: usize) -> String {
        let n = self.indent + extra;
        "    ".repeat(n)
    }
}
