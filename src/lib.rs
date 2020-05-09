#![recursion_limit = "1024"]

extern crate proc_macro;

use heck::SnakeCase;
use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::quote;
use syn::*;

#[proc_macro_derive(DbEnum, attributes(PgType, DieselType, db_rename))]
pub fn derive(input: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(input as DeriveInput);
    let db_type =
        type_from_attrs(&input.attrs, "PgType").unwrap_or(input.ident.to_string().to_snake_case());
    let diesel_mapping =
        type_from_attrs(&input.attrs, "DieselType").unwrap_or(format!("{}Mapping", input.ident));

    let diesel_mapping = Ident::new(diesel_mapping.as_ref(), Span::call_site());
    let quoted = if let Data::Enum(syn::DataEnum {
        variants: data_variants,
        ..
    }) = input.data
    {
        generate_derive_enum_impls(&db_type, &diesel_mapping, &input.ident, &data_variants)
    } else {
        return syn::Error::new(
            Span::call_site(),
            "derive(DbEnum) can only be applied to enums",
        )
        .to_compile_error()
        .into();
    };
    quoted.into()
}

fn type_from_attrs(attrs: &[Attribute], attrname: &str) -> Option<String> {
    for attr in attrs {
        if attr.path.is_ident(attrname) {
            match attr.parse_meta().ok()? {
                Meta::NameValue(MetaNameValue {
                    lit: Lit::Str(lit_str),
                    ..
                }) => return Some(lit_str.value()),
                _ => return None,
            }
        }
    }
    None
}

fn generate_derive_enum_impls(
    db_type: &str,
    diesel_mapping: &Ident,
    enum_ty: &Ident,
    variants: &syn::punctuated::Punctuated<Variant, syn::token::Comma>,
) -> TokenStream {
    let modname = Ident::new(&format!("db_enum_impl_{}", enum_ty), Span::call_site());
    let variant_ids: Vec<proc_macro2::TokenStream> = variants
        .iter()
        .map(|variant| {
            if let Fields::Unit = variant.fields {
                let id = &variant.ident;
                quote! {
                    #enum_ty::#id
                }
            } else {
                panic!("Variants must be fieldless")
            }
        })
        .collect();

    let variants_db: Vec<LitByteStr> = variants
        .iter()
        .map(|variant| {
            let dbname = type_from_attrs(&variant.attrs, "db_rename")
                .unwrap_or(variant.ident.to_string().to_snake_case());
            LitByteStr::new(&dbname.into_bytes(), Span::call_site())
        })
        .collect();

    let variants_rs: &[proc_macro2::TokenStream] = &variant_ids;
    let variants_db: &[LitByteStr] = &variants_db;

    let common_impl = generate_common_impl(diesel_mapping, enum_ty, variants_rs, variants_db);

    let pg_impl = if cfg!(feature = "postgres") {
        generate_postgres_impl(db_type, diesel_mapping, enum_ty, variants_rs, variants_db)
    } else {
        quote! {}
    };
    let mysql_impl = if cfg!(feature = "mysql") {
        generate_mysql_impl(diesel_mapping, enum_ty, variants_rs, variants_db)
    } else {
        quote! {}
    };
    let sqlite_impl = if cfg!(feature = "sqlite") {
        generate_sqlite_impl(diesel_mapping, enum_ty, variants_rs, variants_db)
    } else {
        quote! {}
    };

    let quoted = quote! {
        pub use self::#modname::#diesel_mapping;
        #[allow(non_snake_case)]
        mod #modname {
            #common_impl
            #pg_impl
            #mysql_impl
            #sqlite_impl
        }
    };

    quoted.into()
}

fn generate_common_impl(
    diesel_mapping: &Ident,
    enum_ty: &Ident,
    variants_rs: &[proc_macro2::TokenStream],
    variants_db: &[LitByteStr],
) -> proc_macro2::TokenStream {
    quote! {
        use super::*;
        use diesel::Queryable;
        use diesel::backend::{self, Backend};
        use diesel::expression::AsExpression;
        use diesel::expression::bound::Bound;
        use diesel::row::Row;
        use diesel::sql_types::*;
        use diesel::serialize::{self, ToSql, IsNull, Output};
        use diesel::deserialize::{self, FromSql, FromSqlRow};
        use diesel::query_builder::QueryId;
        use std::io::Write;

        pub struct #diesel_mapping;
        impl QueryId for #diesel_mapping {
            type QueryId = #diesel_mapping;
            const HAS_STATIC_QUERY_ID: bool = true;
        }
        impl NotNull for #diesel_mapping {}
        impl SingleValue for #diesel_mapping {}

        impl AsExpression<#diesel_mapping> for #enum_ty {
            type Expression = Bound<#diesel_mapping, Self>;

            fn as_expression(self) -> Self::Expression {
                Bound::new(self)
            }
        }

        impl AsExpression<Nullable<#diesel_mapping>> for #enum_ty {
            type Expression = Bound<Nullable<#diesel_mapping>, Self>;

            fn as_expression(self) -> Self::Expression {
                Bound::new(self)
            }
        }

        impl<'a> AsExpression<#diesel_mapping> for &'a #enum_ty {
            type Expression = Bound<#diesel_mapping, Self>;

            fn as_expression(self) -> Self::Expression {
                Bound::new(self)
            }
        }

        impl<'a> AsExpression<Nullable<#diesel_mapping>> for &'a #enum_ty {
            type Expression = Bound<Nullable<#diesel_mapping>, Self>;

            fn as_expression(self) -> Self::Expression {
                Bound::new(self)
            }
        }

        impl<'a, 'b> AsExpression<#diesel_mapping> for &'a &'b #enum_ty {
            type Expression = Bound<#diesel_mapping, Self>;

            fn as_expression(self) -> Self::Expression {
                Bound::new(self)
            }
        }

        impl<'a, 'b> AsExpression<Nullable<#diesel_mapping>> for &'a &'b #enum_ty {
            type Expression = Bound<Nullable<#diesel_mapping>, Self>;

            fn as_expression(self) -> Self::Expression {
                Bound::new(self)
            }
        }

        impl<DB: Backend> ToSql<#diesel_mapping, DB> for #enum_ty {
            fn to_sql<W: Write>(&self, out: &mut Output<W, DB>) -> serialize::Result {
                match *self {
                    #(#variants_rs => out.write_all(#variants_db)?,)*
                }
                Ok(IsNull::No)
            }
        }

        impl<DB> ToSql<Nullable<#diesel_mapping>, DB> for #enum_ty
        where
            DB: Backend,
            Self: ToSql<#diesel_mapping, DB>,
        {
            fn to_sql<W: ::std::io::Write>(&self, out: &mut Output<W, DB>) -> serialize::Result {
                ToSql::<#diesel_mapping, DB>::to_sql(self, out)
            }
        }
    }
}

fn generate_postgres_impl(
    db_type: &str,
    diesel_mapping: &Ident,
    enum_ty: &Ident,
    variants_rs: &[proc_macro2::TokenStream],
    variants_db: &[LitByteStr],
) -> proc_macro2::TokenStream {
    quote! {
        mod pg_impl {
            use super::*;
            use diesel::pg::{Pg, PgValue};

            impl HasSqlType<#diesel_mapping> for Pg {
                fn metadata(lookup: &Self::MetadataLookup) -> Self::TypeMetadata {
                    lookup.lookup_type(#db_type)
                }
            }

            impl FromSqlRow<#diesel_mapping, Pg> for #enum_ty {
                fn build_from_row<T: Row<Pg>>(row: &mut T) -> deserialize::Result<Self> {
                    FromSql::<#diesel_mapping, Pg>::from_sql(row.take())
                }
            }

            impl FromSql<#diesel_mapping, Pg> for #enum_ty {
                fn from_sql(raw: Option<PgValue>) -> deserialize::Result<Self> {
                    match raw.as_ref().map(|r| r.as_bytes()) {
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
}

fn generate_mysql_impl(
    diesel_mapping: &Ident,
    enum_ty: &Ident,
    variants_rs: &[proc_macro2::TokenStream],
    variants_db: &[LitByteStr],
) -> proc_macro2::TokenStream {
    quote! {
        mod mysql_impl {
            use super::*;
            use diesel;
            use diesel::mysql::{Mysql, MysqlValue};

            impl HasSqlType<#diesel_mapping> for Mysql {
                fn metadata(_lookup: &Self::MetadataLookup) -> Self::TypeMetadata {
                    diesel::mysql::MysqlTypeMetadata {
                        data_type: diesel::mysql::MysqlType::String,
                        is_unsigned: false
                    }
                }
            }

            impl FromSqlRow<#diesel_mapping, Mysql> for #enum_ty {
                fn build_from_row<T: Row<Mysql>>(row: &mut T) -> deserialize::Result<Self> {
                    FromSql::<#diesel_mapping, Mysql>::from_sql(row.take())
                }
            }

            impl FromSql<#diesel_mapping, Mysql> for #enum_ty {
                fn from_sql(raw: Option<MysqlValue>) -> deserialize::Result<Self> {
                    match raw.as_ref().map(|r| r.as_bytes()) {
                        #(Some(#variants_db) => Ok(#variants_rs),)*
                        Some(v) => Err(format!("Unrecognized enum variant: '{}'",
                                               String::from_utf8_lossy(v)).into()),
                        None => Err("Unexpected null for non-null column".into()),
                    }
                }
            }

            impl Queryable<#diesel_mapping, Mysql> for #enum_ty {
                type Row = Self;

                fn build(row: Self::Row) -> Self {
                    row
                }
            }
        }
    }
}

fn generate_sqlite_impl(
    diesel_mapping: &Ident,
    enum_ty: &Ident,
    variants_rs: &[proc_macro2::TokenStream],
    variants_db: &[LitByteStr],
) -> proc_macro2::TokenStream {
    quote! {
        mod sqlite_impl {
            use super::*;
            use diesel;
            use diesel::sqlite::Sqlite;

            impl HasSqlType<#diesel_mapping> for Sqlite {
                fn metadata(_lookup: &Self::MetadataLookup) -> Self::TypeMetadata {
                    diesel::sqlite::SqliteType::Text
                }
            }

            impl FromSqlRow<#diesel_mapping, Sqlite> for #enum_ty {
                fn build_from_row<T: Row<Sqlite>>(row: &mut T) -> deserialize::Result<Self> {
                    FromSql::<#diesel_mapping, Sqlite>::from_sql(row.take())
                }
            }

            impl FromSql<#diesel_mapping, Sqlite> for #enum_ty {
                fn from_sql(bytes: Option<backend::RawValue<Sqlite>>) -> deserialize::Result<Self> {
                    match bytes.map(|v| v.read_blob()) {
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
}
