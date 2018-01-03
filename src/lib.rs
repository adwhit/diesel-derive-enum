#![recursion_limit = "1024"]

extern crate heck;
extern crate proc_macro;
#[macro_use]
extern crate quote;
extern crate syn;

use proc_macro::TokenStream;
use quote::Tokens;
use syn::*;
use heck::{CamelCase, SnakeCase};

#[proc_macro_derive(PgEnum, attributes(PgType))]
pub fn derive(input: TokenStream) -> TokenStream {
    let input = input.to_string();
    let ast = syn::parse_derive_input(&input).expect("Failed to parse item");

    let attr = &ast.attrs.get(0).expect("Expected attribute: 'PgType'");
    let pg_type = if let MetaItem::NameValue(ref key, Lit::Str(ref pg_type, _)) = attr.value {
        if key != "PgType" {
            panic!("Invalid attribute: {:?}", attr)
        }
        pg_type
    } else {
        panic!("Invalid attribute: {:?}", attr)
    };

    let quoted = match ast.body {
        Body::Enum(ref variants) => pg_enum_impls(pg_type, &ast.ident, variants),
        Body::Struct(_) => panic!("#derive(PgEnum) can only be applied to enums"),
    };
    quoted.parse().unwrap()
}

fn pg_enum_impls(pg_type: &str, enum_: &Ident, variants: &[Variant]) -> Tokens {
    let pg_type_snake = pg_type.to_snake_case();
    let pg_type = Ident::new(pg_type.to_camel_case());
    let modname = Ident::new(format!("pg_enum_impl_{}", pg_type_snake));
    let variants: Vec<Ident> = variants
        .iter()
        .map(|variant| {
            if let VariantData::Unit = variant.data {
                variant.ident.clone()
            } else {
                panic!("Variants must be fieldless")
            }
        })
        .collect();
    let variants_tok: Vec<Tokens> = variants
        .iter()
        .map(|variant| {
            quote! {
                #enum_::#variant
            }
        })
        .collect();
    let variants_snake: Vec<Ident> = variants
        .iter()
        .map(|vid| Ident::new(format!(r#"b"{}""#, vid.as_ref().to_snake_case())))
        .collect();
    let variants: &[Tokens] = &variants_tok;
    let variants_snake: &[Ident] = &variants_snake;
    quote! {
        pub use self::#modname::#pg_type;
        mod #modname {
            use diesel::Queryable;
            use diesel::expression::AsExpression;
            use diesel::expression::bound::Bound;
            use diesel::pg::Pg;
            use diesel::row::Row;
            use diesel::types::*;
            use std::error::Error;
            use std::io::Write;

            pub struct #pg_type;

            impl HasSqlType<#pg_type> for Pg {
                fn metadata(lookup: &Self::MetadataLookup) -> Self::TypeMetadata {
                    lookup.lookup_type(#pg_type_snake)
                }
            }

            impl NotNull for #pg_type {}
            impl SingleValue for #pg_type {}

            impl<'a> AsExpression<#pg_type> for &'a #enum_ {
                type Expression = Bound<#pg_type, &'a #enum_>;

                fn as_expression(self) -> Self::Expression {
                    Bound::new(self)
                }
            }

            impl ToSql<#pg_type, Pg> for #enum_ {
                fn to_sql<W: Write>(
                    &self,
                    out: &mut ToSqlOutput<W, Pg>,
                ) -> Result<IsNull, Box<Error + Send + Sync>> {
                    match *self {
                        #(#variants => out.write_all(#variants_snake)?,)*
                    }
                    Ok(IsNull::No)
                }
            }

            impl FromSqlRow<#pg_type, Pg> for #enum_ {
                fn build_from_row<T: Row<Pg>>(row: &mut T) -> Result<Self, Box<Error + Send + Sync>> {
                    match row.take() {
                        #(Some(#variants_snake) => Ok(#variants),)*
                        Some(_) => Err("Unrecognized enum variant".into()),
                        None => Err("Unexpected null for non-null column".into()),
                    }
                }
            }

            impl Queryable<#pg_type, Pg> for #enum_ {
                type Row = Self;

                fn build(row: Self::Row) -> Self {
                    row
                }
            }
        }
    }
}
