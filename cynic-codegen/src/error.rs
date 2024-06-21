#[derive(Debug, Clone, Default)]
pub struct Errors {
    errors: Vec<syn::Error>,
}

impl Errors {
    pub fn push(&mut self, err: syn::Error) {
        self.errors.push(err);
    }

    pub fn len(&self) -> usize {
        self.errors.len()
    }

    pub fn is_empty(&self) -> bool {
        self.errors.is_empty()
    }

    pub fn into_result<T>(self, ok: T) -> Result<T, Self> {
        match self.is_empty() {
            true => Ok(ok),
            false => Err(self),
        }
    }

    pub fn to_compile_errors(&self) -> proc_macro2::TokenStream {
        let mut rv = proc_macro2::TokenStream::new();
        for err in &self.errors {
            rv.extend(err.to_compile_error());
        }

        rv
    }
}

impl Extend<Errors> for Errors {
    fn extend<T: IntoIterator<Item = Errors>>(&mut self, iter: T) {
        self.errors.extend(iter.into_iter().flat_map(|e| e.errors))
    }
}

impl Extend<syn::Error> for Errors {
    fn extend<T: IntoIterator<Item = syn::Error>>(&mut self, iter: T) {
        self.errors.extend(iter)
    }
}

impl std::iter::FromIterator<Errors> for Errors {
    fn from_iter<T: IntoIterator<Item = Errors>>(iter: T) -> Self {
        let mut rv = Errors { errors: vec![] };
        rv.extend(iter);
        rv
    }
}

impl std::iter::FromIterator<syn::Error> for Errors {
    fn from_iter<T: IntoIterator<Item = syn::Error>>(iter: T) -> Self {
        let mut rv = Errors { errors: vec![] };
        rv.extend(iter);
        rv
    }
}

impl IntoIterator for Errors {
    type Item = syn::Error;

    type IntoIter = std::vec::IntoIter<syn::Error>;

    fn into_iter(self) -> Self::IntoIter {
        self.errors.into_iter()
    }
}

impl From<syn::Error> for Errors {
    fn from(err: syn::Error) -> Errors {
        Errors { errors: vec![err] }
    }
}

impl std::fmt::Display for Errors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for err in &self.errors {
            writeln!(f, "{}", err)?;
        }
        Ok(())
    }
}
