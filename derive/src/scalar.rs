use proc_macro::TokenStream;
use quote::quote;
use syn::{Error, ItemImpl, Type};

use crate::args::{self, RenameTarget};
use crate::utils::{get_crate_name, get_rustdoc, GeneratorResult};

pub fn generate(
    scalar_args: &args::Scalar,
    item_impl: &mut ItemImpl,
) -> GeneratorResult<TokenStream> {
    let crate_name = get_crate_name(scalar_args.internal);
    let self_name = match item_impl.self_ty.as_ref() {
        Type::Path(path) => path
            .path
            .segments
            .last()
            .map(|s| s.ident.to_string())
            .unwrap(),
        _ => return Err(Error::new_spanned(&item_impl.self_ty, "Invalid type").into()),
    };
    let gql_typename = scalar_args
        .name
        .clone()
        .unwrap_or_else(|| RenameTarget::Type.rename(self_name.clone()));

    let desc = if scalar_args.use_type_description {
        quote! { ::std::option::Option::Some(<Self as #crate_name::Description>::description()) }
    } else {
        get_rustdoc(&item_impl.attrs)?
            .map(|s| quote!(::std::option::Option::Some(#s)))
            .unwrap_or_else(|| quote!(::std::option::Option::None))
    };

    let self_ty = &item_impl.self_ty;
    let generic = &item_impl.generics;
    let where_clause = &item_impl.generics.where_clause;
    let expanded = quote! {
        #item_impl

        #[allow(clippy::all, clippy::pedantic)]
        impl #generic #crate_name::Type for #self_ty #where_clause {
            fn type_name() -> ::std::borrow::Cow<'static, ::std::primitive::str> {
                ::std::borrow::Cow::Borrowed(#gql_typename)
            }

            fn create_type_info(registry: &mut #crate_name::registry::Registry) -> ::std::string::String {
                registry.create_type::<#self_ty, _>(|_| #crate_name::registry::MetaType::Scalar {
                    name: ::std::borrow::ToOwned::to_owned(#gql_typename),
                    description: #desc,
                    is_valid: |value| <#self_ty as #crate_name::ScalarType>::is_valid(value),
                })
            }
        }

        #[allow(clippy::all, clippy::pedantic)]
        impl #generic #crate_name::InputValueType for #self_ty #where_clause {
            fn parse(value: ::std::option::Option<#crate_name::Value>) -> #crate_name::InputValueResult<Self> {
                <#self_ty as #crate_name::ScalarType>::parse(value.unwrap_or_default())
            }

            fn to_value(&self) -> #crate_name::Value {
                <#self_ty as #crate_name::ScalarType>::to_value(self)
            }
        }

        #[allow(clippy::all, clippy::pedantic)]
        #[#crate_name::async_trait::async_trait(?Send)]
        impl #generic #crate_name::OutputValueType for #self_ty #where_clause {
            async fn resolve(
                &self,
                _: &#crate_name::ContextSelectionSet<'_>,
                _field: &#crate_name::Positioned<#crate_name::parser::types::Field>
            ) -> #crate_name::ServerResult<#crate_name::Value> {
                ::std::result::Result::Ok(#crate_name::ScalarType::to_value(self))
            }
        }
    };
    Ok(expanded.into())
}
