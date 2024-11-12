use syn::{Attribute, Expr, Field, LitStr, Path};

use crate::renames::RenameAll;

#[derive(Default, Debug)]
pub struct StructAttribute {
    pub default: Option<()>,
    pub rename_all: Option<RenameAll>,
}

impl StructAttribute {
    pub fn from_attrs(attrs: &[Attribute]) -> Self {
        attrs
            .iter()
            .filter(|attr| attr.path().is_ident("deser"))
            .map(|attr| {
                let mut output = StructAttribute::default();
                attr.parse_nested_meta(|meta| {
                    // Note: If adding an attribute in here don't forget to add it to
                    // the merge function below
                    if meta.path.is_ident("default") {
                        output.default = Some(());
                        Ok(())
                    } else if meta.path.is_ident("rename_all") {
                        let value = meta.value()?;
                        let rename = value.parse::<LitStr>()?;
                        output.rename_all = Some(
                            rename
                                .value()
                                .parse()
                                .map_err(|e| syn::Error::new(rename.span(), e))?,
                        );
                        Ok(())
                    } else {
                        Err(meta.error("unsupported attribute"))
                    }
                })
                .unwrap();
                output
            })
            .fold(Self::default(), |acc, inc| acc.merge(inc))
    }

    fn merge(mut self, other: Self) -> Self {
        self.default = self.default.or(other.default);
        self.rename_all = self.rename_all.or(other.rename_all);
        self
    }

    pub fn to_field_defaults(&self) -> FieldAttributes {
        FieldAttributes {
            default: self.default.map(|_| FieldDefault::DefaultImpl),
            ..Default::default()
        }
    }
}

#[derive(Default, Clone)]
pub struct FieldAttributes {
    pub rename: Option<String>,
    pub default: Option<FieldDefault>,
    pub deserialize_with: Option<syn::Path>,
}

#[derive(Clone)]
pub enum FieldDefault {
    DefaultImpl,
    Expression(syn::Expr),
}

impl FieldAttributes {
    pub fn from_field(field: &Field, defaults: FieldAttributes) -> Self {
        field
            .attrs
            .iter()
            .filter(|attr| attr.path().is_ident("deser"))
            .map(|attr| {
                // Note: If adding an attribute in here don't forget to add it to
                // the merge function below
                let mut output = FieldAttributes::default();
                attr.parse_nested_meta(|meta| {
                    if meta.path.is_ident("rename") {
                        let value = meta.value()?;
                        let rename = value.parse::<LitStr>()?;
                        output.rename = Some(rename.value());
                        Ok(())
                    } else if meta.path.is_ident("default") {
                        match meta.value() {
                            Ok(value) => {
                                let expr: Expr = value.parse()?;
                                output.default = Some(FieldDefault::Expression(expr));
                            }
                            Err(_) => output.default = Some(FieldDefault::DefaultImpl),
                        }
                        Ok(())
                    } else if meta.path.is_ident("with") || meta.path.is_ident("deserialize_with") {
                        let value = meta.value()?;
                        let path = value.parse::<Path>()?;
                        output.deserialize_with = Some(path);
                        Ok(())
                    } else {
                        Err(meta.error("unsupported attribute"))
                    }
                })
                .unwrap();
                output
            })
            .fold(defaults, |acc, inc| acc.merge(inc))
    }

    fn merge(mut self, other: Self) -> Self {
        // Note that other takes precedence here so that our defaults work correctly.
        self.rename = other.rename.or(self.rename);
        self.default = other.default.or(self.default);
        self.deserialize_with = other.deserialize_with.or(self.deserialize_with);
        self
    }
}
