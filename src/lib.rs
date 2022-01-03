#![recursion_limit = "1024"]

extern crate proc_macro;

use heck::{ToKebabCase, ToLowerCamelCase, ToShoutySnakeCase, ToSnakeCase, ToUpperCamelCase};
use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::quote;
use syn::*;

/// Implement necessary traits for adding a database enum as a new sql type.
///
/// # Attributes
///
/// ## Type attributes
///
/// * `#[DieselExistingType = "crate::schema::sql_types::NewEnum"]` specifies the name for the corresponding diesel type that was already created by the diesel CLI. If omitted, uses `crate::schema::sql_types::EnumName`.
/// * `#[DieselType = "NewEnumMapping"]` specifies the name for the diesel type to create for Mysql or Sqlite. If omitted, uses the name + `Mapping`.
/// * `#[DbValueStyle = "snake_case"]` specifies a renaming style from each of the rust enum variants to each of the database variants. Either `camelCase`, `kebab-case`, `PascalCase`, `SCREAMING_SNAKE_CASE`, `snake_case`, `verbatim`. If omitted, uses `snake_case`.
///
/// ## Variant attributes
///
/// * `#[db_rename = "variant"]` specifies the db name for a specific variant.
#[proc_macro_derive(
    DbEnum,
    attributes(DieselType, DieselExistingType, DbValueStyle, db_rename)
)]
pub fn derive(input: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(input as DeriveInput);
    let diesel_existing_mapping = type_from_attrs(&input.attrs, "DieselExistingType")
        .unwrap_or(format!("crate::schema::sql_types::{}", input.ident));
    let new_diesel_mapping =
        type_from_attrs(&input.attrs, "DieselType").unwrap_or(format!("{}Mapping", input.ident));

    // Maintain backwards compatibility by defaulting to snake case.
    let case_style =
        type_from_attrs(&input.attrs, "DbValueStyle").unwrap_or("snake_case".to_string());
    let case_style = CaseStyle::from_string(&case_style);

    let diesel_existing_mapping: proc_macro2::TokenStream =
        diesel_existing_mapping.parse().unwrap();
    let new_diesel_mapping = Ident::new(new_diesel_mapping.as_ref(), Span::call_site());
    let quoted = if let Data::Enum(syn::DataEnum {
        variants: data_variants,
        ..
    }) = input.data
    {
        generate_derive_enum_impls(
            &diesel_existing_mapping,
            &new_diesel_mapping,
            case_style,
            &input.ident,
            &data_variants,
        )
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

/// Defines the casing for the database representation.  Follows serde naming convention.
#[derive(Copy, Clone, Debug, PartialEq)]
enum CaseStyle {
    Camel,
    Kebab,
    Pascal,
    ScreamingSnake,
    Snake,
    Verbatim,
}

impl CaseStyle {
    fn from_string(name: &str) -> Self {
        match name {
            "camelCase" => CaseStyle::Camel,
            "kebab-case" => CaseStyle::Kebab,
            "PascalCase" => CaseStyle::Pascal,
            "SCREAMING_SNAKE_CASE" => CaseStyle::ScreamingSnake,
            "snake_case" => CaseStyle::Snake,
            "verbatim" | "verbatimcase" => CaseStyle::Verbatim,
            s => panic!("unsupported casing: `{}`", s),
        }
    }
}

fn generate_derive_enum_impls(
    diesel_existing_mapping: &proc_macro2::TokenStream,
    new_diesel_mapping: &Ident,
    case_style: CaseStyle,
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

    let variants_db: Vec<String> = variants
        .iter()
        .map(|variant| {
            type_from_attrs(&variant.attrs, "db_rename")
                .unwrap_or(stylize_value(&variant.ident.to_string(), case_style))
        })
        .collect();
    let variants_db_bytes: Vec<LitByteStr> = variants_db
        .iter()
        .map(|variant_str| LitByteStr::new(variant_str.as_bytes(), Span::call_site()))
        .collect();

    let common = generate_common(enum_ty, &variant_ids, &variants_db, &variants_db_bytes);
    let (common_diesel_mapping, common_diesel_mapping_use) =
        if cfg!(feature = "mysql") || cfg!(feature = "sqlite") {
            let new_diesel_mapping_impl = generate_common_diesel_mapping(new_diesel_mapping);
            let common_impls_on_new_diesel_mapping =
                generate_common_impls(&quote! { #new_diesel_mapping }, enum_ty);
            (
                quote! {
                    #new_diesel_mapping_impl
                    #common_impls_on_new_diesel_mapping
                },
                quote! {
                    pub use self::#modname::#new_diesel_mapping;
                },
            )
        } else {
            (quote! {}, quote! {})
        };

    let pg_impl = if cfg!(feature = "postgres") {
        let common_impls_on_existing_diesel_mapping =
            generate_common_impls(diesel_existing_mapping, enum_ty);
        let postgres_impl = generate_postgres_impl(diesel_existing_mapping, enum_ty);
        quote! {
            #common_impls_on_existing_diesel_mapping
            #postgres_impl
        }
    } else {
        quote! {}
    };
    let mysql_impl = if cfg!(feature = "mysql") {
        generate_mysql_impl(new_diesel_mapping, enum_ty)
    } else {
        quote! {}
    };
    let sqlite_impl = if cfg!(feature = "sqlite") {
        generate_sqlite_impl(new_diesel_mapping, enum_ty)
    } else {
        quote! {}
    };

    let imports = quote! {
        use super::*;
        use diesel::{
            backend::{self, Backend},
            deserialize::{self, FromSql},
            expression::{bound::Bound, AsExpression},
            query_builder::{bind_collector::RawBytesBindCollector, QueryId},
            row::Row,
            serialize::{self, IsNull, Output, ToSql},
            sql_types::*,
            Queryable,
        };
        use std::io::Write;
    };

    let quoted = quote! {
        #common_diesel_mapping_use
        #[allow(non_snake_case)]
        mod #modname {
            #imports

            #common
            #common_diesel_mapping
            #pg_impl
            #mysql_impl
            #sqlite_impl
        }
    };

    quoted.into()
}

fn stylize_value(value: &str, style: CaseStyle) -> String {
    match style {
        CaseStyle::Camel => value.to_lower_camel_case(),
        CaseStyle::Kebab => value.to_kebab_case(),
        CaseStyle::Pascal => value.to_upper_camel_case(),
        CaseStyle::ScreamingSnake => value.to_shouty_snake_case(),
        CaseStyle::Snake => value.to_snake_case(),
        CaseStyle::Verbatim => value.to_string(),
    }
}

fn generate_common(
    enum_ty: &Ident,
    variants_rs: &[proc_macro2::TokenStream],
    variants_db: &[String],
    variants_db_bytes: &[LitByteStr],
) -> proc_macro2::TokenStream {
    quote! {
        fn db_str_representation(e: &#enum_ty) -> &'static str {
            match *e {
                #(#variants_rs => #variants_db,)*
            }
        }

        fn from_db_binary_representation(bytes: &[u8]) -> deserialize::Result<#enum_ty> {
            match bytes {
                #(#variants_db_bytes => Ok(#variants_rs),)*
                v => Err(format!("Unrecognized enum variant: '{}'",
                    String::from_utf8_lossy(v)).into()),
            }
        }
    }
}

fn generate_common_diesel_mapping(new_diesel_mapping: &Ident) -> proc_macro2::TokenStream {
    quote! {
        #[derive(SqlType, Clone)]
        #[diesel(mysql_type(name = "Enum"))]
        #[diesel(sqlite_type(name = "Text"))]
        pub struct #new_diesel_mapping;
    }
}

fn generate_common_impls(
    diesel_mapping: &proc_macro2::TokenStream,
    enum_ty: &Ident,
) -> proc_macro2::TokenStream {
    quote! {
        impl QueryId for #diesel_mapping {
            type QueryId = #diesel_mapping;
            const HAS_STATIC_QUERY_ID: bool = true;
        }

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

        impl<DB> ToSql<Nullable<#diesel_mapping>, DB> for #enum_ty
        where
            DB: Backend,
            Self: ToSql<#diesel_mapping, DB>,
        {
            fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, DB>) -> serialize::Result {
                ToSql::<#diesel_mapping, DB>::to_sql(self, out)
            }
        }
    }
}

fn generate_postgres_impl(
    diesel_mapping: &proc_macro2::TokenStream,
    enum_ty: &Ident,
) -> proc_macro2::TokenStream {
    quote! {
        mod pg_impl {
            use super::*;
            use diesel::pg::{Pg, PgValue};

            impl Clone for #diesel_mapping {
                fn clone(&self) -> Self {
                    #diesel_mapping
                }
            }

            impl FromSql<#diesel_mapping, Pg> for #enum_ty {
                fn from_sql(raw: PgValue) -> deserialize::Result<Self> {
                    from_db_binary_representation(raw.as_bytes())
                }
            }

            impl ToSql<#diesel_mapping, Pg> for #enum_ty
            {
                fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> serialize::Result {
                    out.write_all(db_str_representation(self).as_bytes())?;
                    Ok(IsNull::No)
                }
            }

            impl Queryable<#diesel_mapping, Pg> for #enum_ty {
                type Row = Self;

                fn build(row: Self::Row) -> deserialize::Result<Self> {
                    Ok(row)
                }
            }
        }
    }
}

fn generate_mysql_impl(diesel_mapping: &Ident, enum_ty: &Ident) -> proc_macro2::TokenStream {
    quote! {
        mod mysql_impl {
            use super::*;
            use diesel;
            use diesel::mysql::{Mysql, MysqlValue};

            impl FromSql<#diesel_mapping, Mysql> for #enum_ty {
                fn from_sql(raw: MysqlValue) -> deserialize::Result<Self> {
                    from_db_binary_representation(raw.as_bytes())
                }
            }

            impl ToSql<#diesel_mapping, Mysql> for #enum_ty
            {
                fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Mysql>) -> serialize::Result {
                    out.write_all(db_str_representation(self).as_bytes())?;
                    Ok(IsNull::No)
                }
            }

            impl Queryable<#diesel_mapping, Mysql> for #enum_ty {
                type Row = Self;

                fn build(row: Self::Row) -> deserialize::Result<Self> {
                    Ok(row)
                }
            }
        }
    }
}

fn generate_sqlite_impl(diesel_mapping: &Ident, enum_ty: &Ident) -> proc_macro2::TokenStream {
    quote! {
        mod sqlite_impl {
            use super::*;
            use diesel;
            use diesel::sql_types;
            use diesel::sqlite::Sqlite;

            impl FromSql<#diesel_mapping, Sqlite> for #enum_ty {
                fn from_sql(value: backend::RawValue<Sqlite>) -> deserialize::Result<Self> {
                    let bytes = <Vec<u8> as FromSql<sql_types::Binary, Sqlite>>::from_sql(value)?;
                    from_db_binary_representation(bytes.as_slice())
                }
            }

            impl ToSql<#diesel_mapping, Sqlite> for #enum_ty {
                fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Sqlite>) -> serialize::Result {
                    <str as ToSql<sql_types::Text, Sqlite>>::to_sql(db_str_representation(self), out)
                }
            }

            impl Queryable<#diesel_mapping, Sqlite> for #enum_ty {
                type Row = Self;

                fn build(row: Self::Row) -> deserialize::Result<Self> {
                    Ok(row)
                }
            }
        }
    }
}
