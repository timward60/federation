use std::fmt;

use crate::format::{format_directives, Displayable, Formatter, Style};

use crate::query::ast::*;
use crate::query::refs::{
    FieldRef, FragmentSpreadRef, InlineFragmentRef, SelectionRef, SelectionSetRef,
};

impl<'a> Document<'a> {
    /// Format a document according to style
    pub fn format(&self, style: &Style) -> String {
        let mut formatter = Formatter::new(style);
        self.display(&mut formatter);
        formatter.into_string()
    }
}

fn to_string<T: Displayable>(v: &T) -> String {
    let style = Style::default();
    let mut formatter = Formatter::new(&style);
    v.display(&mut formatter);
    formatter.into_string()
}

impl<'a> Displayable for Document<'a> {
    fn display(&self, f: &mut Formatter) {
        for item in &self.definitions {
            item.display(f);
        }
    }
}

impl<'a> Displayable for Definition<'a> {
    fn display(&self, f: &mut Formatter) {
        match *self {
            Definition::SelectionSet(ref s) => s.display(f),
            Definition::Operation(ref op) => op.display(f),
            Definition::Fragment(ref frag) => frag.display(f),
        }
    }
}

impl<'a> Displayable for FragmentDefinition<'a> {
    fn display(&self, f: &mut Formatter) {
        f.margin();
        f.indent();
        f.write("fragment ");
        f.write(self.name.as_ref());
        f.write(" on ");
        f.write(self.type_condition);
        format_directives(&self.directives, f);
        f.write(" ");
        f.start_block();
        for item in &self.selection_set.items {
            item.display(f);
        }
        f.end_block();
    }
}

impl<'a> Displayable for SelectionSet<'a> {
    #[inline]
    fn display(&self, f: &mut Formatter) {
        f.margin();
        f.indent();
        f.start_block();
        for item in &self.items {
            item.display(f);
        }
        f.end_block();
    }
}

impl<'a> Displayable for SelectionSetRef<'a> {
    #[inline]
    fn display(&self, f: &mut Formatter) {
        f.margin();
        f.indent();
        f.start_block();
        for item in &self.items {
            item.display(f);
        }
        f.end_block();
    }
}

impl<'a> Displayable for Selection<'a> {
    fn display(&self, f: &mut Formatter) {
        match self {
            Selection::Field(fld) => fld.display(f),
            Selection::InlineFragment(frag) => frag.display(f),
            Selection::FragmentSpread(frag) => frag.display(f),
        }
    }
}

impl<'a> Displayable for SelectionRef<'a> {
    fn display(&self, f: &mut Formatter) {
        match self {
            SelectionRef::Ref(r) => r.display(f),
            SelectionRef::Field(field) => field.display(f),
            SelectionRef::FieldRef(fr) => fr.display(f),
            SelectionRef::InlineFragmentRef(inline) => inline.display(f),
            SelectionRef::FragmentSpreadRef(fsr) => fsr.display(f),
        }
    }
}

fn format_arguments<'a>(arguments: &[(Txt<'a>, Value<'a>)], f: &mut Formatter) {
    if !arguments.is_empty() {
        f.write("(");
        f.write(arguments[0].0.as_ref());
        f.write(": ");
        arguments[0].1.display(f);
        for arg in &arguments[1..] {
            f.write(", ");
            f.write(arg.0.as_ref());
            f.write(": ");
            arg.1.display(f);
        }
        f.write(")");
    }
}

macro_rules! field_impl {
    ($self:ident, $f:ident) => {
        $f.indent();
        if let Some(ref alias) = $self.alias {
            $f.write(alias.as_ref());
            $f.write(": ");
        }
        $f.write($self.name.as_ref());
        format_arguments(&$self.arguments, $f);
        format_directives(&$self.directives, $f);
        if !$self.selection_set.items.is_empty() {
            $f.write(" ");
            $f.start_block();
            for item in &$self.selection_set.items {
                item.display($f);
            }
            $f.end_block();
        } else {
            $f.endline();
        }
    };
}

impl<'a> Displayable for Field<'a> {
    fn display(&self, f: &mut Formatter) {
        field_impl!(self, f);
    }
}

impl<'a> Displayable for FieldRef<'a> {
    fn display(&self, f: &mut Formatter) {
        field_impl!(self, f);
    }
}

impl<'a> Displayable for OperationDefinition<'a> {
    fn display(&self, f: &mut Formatter) {
        f.margin();
        f.indent();
        f.write(self.kind.as_str());
        if let Some(ref name) = self.name {
            f.write(" ");
            f.write(name.as_ref());
            if !self.variable_definitions.is_empty() {
                f.write("(");
                self.variable_definitions[0].display(f);
                for var in &self.variable_definitions[1..] {
                    f.write(", ");
                    var.display(f);
                }
                f.write(")");
            }
        }
        format_directives(&self.directives, f);
        f.write(" ");
        f.start_block();
        for item in &self.selection_set.items {
            item.display(f);
        }
        f.end_block();
    }
}

impl<'a> Displayable for VariableDefinition<'a> {
    fn display(&self, f: &mut Formatter) {
        f.write("$");
        f.write(self.name.as_ref());
        f.write(": ");
        self.var_type.display(f);
        if let Some(ref default) = self.default_value {
            f.write(" = ");
            default.display(f);
        }
    }
}

impl<'a> Displayable for Type<'a> {
    fn display(&self, f: &mut Formatter) {
        match self {
            Type::NamedType(name) => f.write(name.as_ref()),
            Type::ListType(typ) => {
                f.write("[");
                typ.display(f);
                f.write("]");
            }
            Type::NonNullType(typ) => {
                typ.display(f);
                f.write("!");
            }
        }
    }
}

impl<'a> Displayable for Value<'a> {
    fn display(&self, f: &mut Formatter) {
        match self {
            Value::Variable(name) => {
                f.write("$");
                f.write(name.as_ref());
            }
            Value::Int(num) => f.write(&format!("{}", num)),
            Value::Float(val) => f.write(&format!("{}", val)),
            Value::String(val) => f.write_quoted(val),
            Value::Boolean(true) => f.write("true"),
            Value::Boolean(false) => f.write("false"),
            Value::Null => f.write("null"),
            Value::Enum(name) => f.write(name.as_ref()),
            Value::List(items) => {
                f.write("[");
                if !items.is_empty() {
                    items[0].display(f);
                    for item in &items[1..] {
                        f.write(", ");
                        item.display(f);
                    }
                }
                f.write("]");
            }
            Value::Object(items) => {
                f.write("{");
                let mut first = true;
                for (name, value) in items.iter() {
                    if first {
                        first = false;
                    } else {
                        f.write(", ");
                    }
                    f.write(name.as_ref());
                    f.write(": ");
                    value.display(f);
                }
                f.write("}");
            }
        }
    }
}

macro_rules! inline_fragment_impl {
    ($self:ident, $f:ident) => {
        $f.indent();
        $f.write("...");
        if let Some(ref cond) = $self.type_condition {
            $f.write(" on ");
            $f.write(cond);
        }
        format_directives(&$self.directives, $f);
        $f.write(" ");
        $f.start_block();
        for item in &$self.selection_set.items {
            item.display($f);
        }
        $f.end_block();
    };
}

impl<'a> Displayable for InlineFragment<'a> {
    fn display(&self, f: &mut Formatter) {
        inline_fragment_impl!(self, f);
    }
}

impl<'a> Displayable for InlineFragmentRef<'a> {
    fn display(&self, f: &mut Formatter) {
        inline_fragment_impl!(self, f);
    }
}

impl<'a> Displayable for FragmentSpread<'a> {
    fn display(&self, f: &mut Formatter) {
        f.indent();
        f.write("...");
        f.write(self.fragment_name.as_ref());
        format_directives(&self.directives, f);
        f.endline();
    }
}

impl Displayable for FragmentSpreadRef {
    fn display(&self, f: &mut Formatter) {
        f.indent();
        f.write("...");
        f.write(self.name.as_ref());
        f.endline();
    }
}

impl<'a> Displayable for Directive<'a> {
    fn display(&self, f: &mut Formatter) {
        f.write("@");
        f.write(self.name.as_ref());
        format_arguments(self.arguments.as_slice(), f);
    }
}

impl_display!(
    'a
    Document,
    Definition,
    OperationDefinition,
    FragmentDefinition,
    SelectionSet,
    Field,
    VariableDefinition,
    Type,
    Value,
    InlineFragment,
    FragmentSpread,
    Directive,
    SelectionRef,
    SelectionSetRef,
    FieldRef,
    InlineFragmentRef,
);
