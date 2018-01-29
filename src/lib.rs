#![recursion_limit = "1024"]

extern crate heck;
extern crate proc_macro;
#[macro_use]
extern crate quote;
extern crate syn;

use proc_macro::TokenStream;
use quote::Tokens;
use syn::*;
use heck::SnakeCase;

#[proc_macro_derive(PgEnum, attributes(PgType, DieselType, pg_rename))]
pub fn derive(input: TokenStream) -> TokenStream {
    let input = input.to_string();
    let ast = syn::parse_derive_input(&input).expect("Failed to parse item");
    let pg_type =
        type_from_attrs(&ast.attrs, "PgType").unwrap_or(ast.ident.as_ref().to_snake_case());
    let diesel_type = type_from_attrs(&ast.attrs, "DieselType")
        .unwrap_or(format!("{}Mapping", ast.ident.as_ref()));
    let diesel_type = Ident::new(diesel_type);

    let quoted = match ast.body {
        Body::Enum(ref variants) => pg_enum_impls(&pg_type, &diesel_type, &ast.ident, variants),
        Body::Struct(_) => panic!("#derive(PgEnum) can only be applied to enums"),
    };
    quoted.parse().unwrap()
}

fn type_from_attrs(attrs: &[Attribute], attrname: &str) -> Option<String> {
    for attr in attrs {
        if let MetaItem::NameValue(ref key, Lit::Str(ref type_, _)) = attr.value {
            if key == attrname {
                return Some(type_.clone());
            }
        }
    }
    None
}

fn pg_enum_impls(
    pg_type: &str,
    diesel_type: &Ident,
    enum_: &Ident,
    variants: &[Variant],
) -> Tokens {
    let modname = Ident::new(format!("pg_enum_impl_{}", enum_.as_ref()));
    let variant_ids: Vec<Tokens> = variants
        .iter()
        .map(|variant| {
            if let VariantData::Unit = variant.data {
                let id = &variant.ident;
                quote! {
                    #enum_::#id
                }
            } else {
                panic!("Variants must be fieldless")
            }
        })
        .collect();
    let variants_pg: Vec<Ident> = variants
        .iter()
        .map(|variant| {
            let pgname = type_from_attrs(&variant.attrs, "pg_rename")
                .unwrap_or(variant.ident.as_ref().to_snake_case());
            Ident::new(format!(r#"b"{}""#, pgname))
        })
        .collect();
    let variants: &[Tokens] = &variant_ids;
    let variants_pg: &[Ident] = &variants_pg;
    quote! {
        pub use self::#modname::#diesel_type;
        #[allow(non_snake_case)]
        mod #modname {
            use diesel::Queryable;
            use diesel::expression::AsExpression;
            use diesel::expression::bound::Bound;
            use diesel::pg::Pg;
            use diesel::row::Row;
            use diesel::sql_types::*;
            use diesel::serialize::{self, ToSql, IsNull, Output};
            use diesel::deserialize::{self, FromSqlRow};
            use std::io::Write;

            pub struct #diesel_type;

            impl HasSqlType<#diesel_type> for Pg {
                fn metadata(lookup: &Self::MetadataLookup) -> Self::TypeMetadata {
                    lookup.lookup_type(#pg_type)
                }
            }

            impl NotNull for #diesel_type {}
            impl SingleValue for #diesel_type {}

            impl AsExpression<#diesel_type> for #enum_ {
                type Expression = Bound<#diesel_type, #enum_>;

                fn as_expression(self) -> Self::Expression {
                    Bound::new(self)
                }
            }

            impl AsExpression<Nullable<#diesel_type>> for #enum_ {
                type Expression = Bound<Nullable<#diesel_type>, #enum_>;

                fn as_expression(self) -> Self::Expression {
                    Bound::new(self)
                }
            }

            impl<'a> AsExpression<#diesel_type> for &'a #enum_ {
                type Expression = Bound<#diesel_type, &'a #enum_>;

                fn as_expression(self) -> Self::Expression {
                    Bound::new(self)
                }
            }

            impl<'a> AsExpression<Nullable<#diesel_type>> for &'a #enum_ {
                type Expression = Bound<Nullable<#diesel_type>, &'a #enum_>;

                fn as_expression(self) -> Self::Expression {
                    Bound::new(self)
                }
            }

            impl ToSql<#diesel_type, Pg> for #enum_ {
                fn to_sql<W: Write>(&self, out: &mut Output<W, Pg>) -> serialize::Result {
                    match *self {
                        #(#variants => out.write_all(#variants_pg)?,)*
                    }
                    Ok(IsNull::No)
                }
            }

            impl FromSqlRow<#diesel_type, Pg> for #enum_ {
                fn build_from_row<T: Row<Pg>>(row: &mut T) -> deserialize::Result<Self> {
                    match row.take() {
                        #(Some(#variants_pg) => Ok(#variants),)*
                        Some(v) => Err(format!("Unrecognized enum variant: '{}'",
                                               String::from_utf8_lossy(v)).into()),
                        None => Err("Unexpected null for non-null column".into()),
                    }
                }
            }

            impl Queryable<#diesel_type, Pg> for #enum_ {
                type Row = Self;

                fn build(row: Self::Row) -> Self {
                    row
                }
            }
        }
    }
}
