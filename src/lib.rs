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

#[proc_macro_derive(PgEnum, attributes(PgType, DieselType, db_rename))]
pub fn derive(input: TokenStream) -> TokenStream {
    let input = input.to_string();
    let ast = syn::parse_derive_input(&input).expect("Failed to parse item");
    let db_type =
        type_from_attrs(&ast.attrs, "PgType").unwrap_or(ast.ident.as_ref().to_snake_case());
    let diesel_mapping = type_from_attrs(&ast.attrs, "DieselType")
        .unwrap_or(format!("{}Mapping", ast.ident.as_ref()));
    let diesel_mapping = Ident::new(diesel_mapping);

    let quoted = if let Body::Enum(ref variants) = ast.body {
        generate_derive_enum_impls(&db_type, &diesel_mapping, &ast.ident, variants)
    } else {
        panic!("#derive(PgEnum) can only be applied to enums")
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

fn generate_derive_enum_impls(
    db_type: &str,
    diesel_mapping: &Ident,
    enum_ty: &Ident,
    variants: &[Variant],
) -> Tokens {
    let modname = Ident::new(format!("db_enum_impl_{}", enum_ty.as_ref()));
    let variant_ids: Vec<Tokens> = variants
        .iter()
        .map(|variant| {
            if let VariantData::Unit = variant.data {
                let id = &variant.ident;
                quote! {
                    #enum_ty::#id
                }
            } else {
                panic!("Variants must be fieldless")
            }
        })
        .collect();
    let variants_db: Vec<Ident> = variants
        .iter()
        .map(|variant| {
            let dbname = type_from_attrs(&variant.attrs, "db_rename")
                .unwrap_or(variant.ident.as_ref().to_snake_case());
            Ident::new(format!(r#"b"{}""#, dbname))
        })
        .collect();
    let variants_rs: &[Tokens] = &variant_ids;
    let variants_db: &[Ident] = &variants_db;

    let common_impl = generate_common_impl(diesel_mapping, enum_ty);
    let pg_impl = generate_postgres_impl(db_type, diesel_mapping, enum_ty, variants_rs, variants_db);
    let sqlite_impl = generate_sqlite_impl(diesel_mapping, enum_ty, variants_rs, variants_db);
    quote! {
        pub use self::#modname::#diesel_mapping;
        #[allow(non_snake_case)]
        mod #modname {
            #common_impl
            #pg_impl
            #sqlite_impl
        }
    }
}

fn generate_common_impl(
    diesel_mapping: &Ident,
    enum_ty: &Ident,
) -> Tokens {
    quote! {
        use diesel::Queryable;
        use diesel::expression::AsExpression;
        use diesel::expression::bound::Bound;
        use diesel::row::Row;
        use diesel::sql_types::*;
        use diesel::serialize::{self, ToSql, IsNull, Output};
        use diesel::deserialize::{self, FromSqlRow};
        use std::io::Write;

        pub struct #diesel_mapping;

        impl NotNull for #diesel_mapping {}
        impl SingleValue for #diesel_mapping {}

        impl AsExpression<#diesel_mapping> for #enum_ty {
            type Expression = Bound<#diesel_mapping, #enum_ty>;

            fn as_expression(self) -> Self::Expression {
                Bound::new(self)
            }
        }

        impl AsExpression<Nullable<#diesel_mapping>> for #enum_ty {
            type Expression = Bound<Nullable<#diesel_mapping>, #enum_ty>;

            fn as_expression(self) -> Self::Expression {
                Bound::new(self)
            }
        }

        impl<'a> AsExpression<#diesel_mapping> for &'a #enum_ty {
            type Expression = Bound<#diesel_mapping, &'a #enum_ty>;

            fn as_expression(self) -> Self::Expression {
                Bound::new(self)
            }
        }

        impl<'a> AsExpression<Nullable<#diesel_mapping>> for &'a #enum_ty {
            type Expression = Bound<Nullable<#diesel_mapping>, &'a #enum_ty>;

            fn as_expression(self) -> Self::Expression {
                Bound::new(self)
            }
        }
    }
}

fn generate_postgres_impl(
    db_type: &str,
    diesel_mapping: &Ident,
    enum_ty: &Ident,
    variants_rs: &[Tokens],
    variants_db: &[Ident],
) -> Tokens {
    quote! {
        use diesel::pg::Pg;

        impl HasSqlType<#diesel_mapping> for Pg {
            fn metadata(lookup: &Self::MetadataLookup) -> Self::TypeMetadata {
                lookup.lookup_type(#db_type)
            }
        }

        impl ToSql<#diesel_mapping, Pg> for #enum_ty {
            fn to_sql<W: Write>(&self, out: &mut Output<W, Pg>) -> serialize::Result {
                match *self {
                    #(#variants_rs => out.write_all(#variants_db)?,)*
                }
                Ok(IsNull::No)
            }
        }

        impl FromSqlRow<#diesel_mapping, Pg> for #enum_ty {
            fn build_from_row<T: Row<Pg>>(row: &mut T) -> deserialize::Result<Self> {
                match row.take() {
                    #(Some(#variants_db) => Ok(#variants_rs),)*
                    Some(v) => Err(format!("Unrecognized enum variant: '{}'",
                                           String::from_utf8_lossy(v)).into()),
                    None => Err("Unexpected null for non-null column".into()),
                }
            }
        }

        impl Queryable<#diesel_mapping, Pg> for #enum_ty {
            type Row = Self;

            fn build(row: Self::Row) -> Self {
                row
            }
        }
    }
}

fn generate_sqlite_impl(
    diesel_mapping: &Ident,
    enum_ty: &Ident,
    variants_rs: &[Tokens],
    variants_db: &[Ident],
) -> Tokens {
    quote! {
        use diesel::sqlite::Sqlite;

        impl HasSqlType<#diesel_mapping> for Sqlite {
            fn metadata(_lookup: &Self::MetadataLookup) -> Self::TypeMetadata {
                diesel::sqlite::SqliteType::Text
            }
        }

        impl ToSql<#diesel_mapping, Sqlite> for #enum_ty {
            fn to_sql<W: Write>(&self, out: &mut Output<W, Sqlite>) -> serialize::Result {
                match *self {
                    #(#variants_rs => out.write_all(#variants_db)?,)*
                }
                Ok(IsNull::No)
            }
        }

        impl FromSqlRow<#diesel_mapping, Sqlite> for #enum_ty {
            fn build_from_row<T: Row<Sqlite>>(row: &mut T) -> deserialize::Result<Self> {
                match row.take().map(|v| v.read_blob()) {
                    #(Some(#variants_db) => Ok(#variants_rs),)*
                    Some(blob) => Err(format!("Unexpected variant: {}", String::from_utf8_lossy(blob)).into()),
                    None => Err("Unexpected null for non-null column".into()),
                }
            }
        }

        impl Queryable<#diesel_mapping, Sqlite> for #enum_ty {
            type Row = Self;

            fn build(row: Self::Row) -> Self {
                row
            }
        }
    }
}
