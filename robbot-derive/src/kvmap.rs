use std::collections::HashMap;

use syn::parse::{Parse, ParseStream, Result};
use syn::punctuated::Punctuated;
use syn::{Expr, Token};

/// A key-value map.
///
/// A key-value map following the format of `left = right` using the tokens `<expr> = `<expr>``.
#[derive(Clone, Debug)]
pub struct KeyValueMap {
    map: HashMap<String, Option<Expr>>,
}

impl KeyValueMap {
    pub fn get(&self, key: &str) -> Option<&Expr> {
        self.map.get(key).map(|expr| expr.as_ref()).flatten()
    }
}

impl Parse for KeyValueMap {
    fn parse(input: ParseStream) -> Result<Self> {
        let args = Punctuated::<Expr, Token![,]>::parse_terminated(input)?;

        let mut map = HashMap::new();

        for arg in args {
            match arg {
                Expr::Assign(expr) => {
                    let ident = match &*expr.left {
                        Expr::Path(expr) => expr.path.segments.first().unwrap().ident.clone(),
                        _ => panic!("Invalid expr: {:?}, expected Expr::Path", expr.left),
                    };

                    let key = ident.to_string();

                    if map.contains_key(&key) {
                        panic!("Key {} already exists", key);
                    }

                    map.insert(key, Some(*expr.right));
                }
                _ => panic!("Invalid expr: {:?}, expected Expr::Assign", arg),
            }
        }

        Ok(Self { map })
    }
}
