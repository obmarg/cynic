/// A simplified rust type structure
#[derive(Debug, PartialEq)]
pub enum RustType<'a> {
    Optional(&'a syn::Type),
    List(&'a syn::Type),
    Box(&'a syn::Type),
    SimpleType,
    Unknown,
}

#[allow(clippy::cmp_owned)]
pub fn parse_rust_type(ty: &'_ syn::Type) -> RustType<'_> {
    if let syn::Type::Path(type_path) = ty {
        if let Some(last_segment) = type_path.path.segments.last() {
            if let syn::PathArguments::None = last_segment.arguments {
                return RustType::SimpleType;
            }

            match last_segment.ident.to_string().as_ref() {
                "Box" | "Arc" | "Rc" => {
                    if let Some(inner_type) = extract_generic_argument(last_segment) {
                        return RustType::Box(inner_type);
                    }
                }
                "Option" => {
                    if let Some(inner_type) = extract_generic_argument(last_segment) {
                        return RustType::Optional(inner_type);
                    }
                }
                "Vec" => {
                    if let Some(inner_type) = extract_generic_argument(last_segment) {
                        return RustType::List(inner_type);
                    }
                }
                _ => {}
            }
            return RustType::Unknown;
        }
    }

    RustType::Unknown
}

/// Takes a PathSegment like `Vec<T>` and extracts the `T`
fn extract_generic_argument(segment: &syn::PathSegment) -> Option<&syn::Type> {
    if let syn::PathArguments::AngleBracketed(angle_bracketed) = &segment.arguments {
        for arg in &angle_bracketed.args {
            if let syn::GenericArgument::Type(inner_type) = arg {
                return Some(inner_type);
            }
        }
    }

    None
}
