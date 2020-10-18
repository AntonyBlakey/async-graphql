use darling::ast::Data;
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{Error, LitInt};

use crate::args::{self, RenameTarget};
use crate::utils::{get_crate_name, get_rustdoc, GeneratorResult};

pub fn generate(object_args: &args::MergedObject) -> GeneratorResult<TokenStream> {
    let crate_name = get_crate_name(object_args.internal);
    let ident = &object_args.ident;
    let extends = object_args.extends;
    let gql_typename = object_args
        .name
        .clone()
        .unwrap_or_else(|| RenameTarget::Type.rename(ident.to_string()));

    let desc = get_rustdoc(&object_args.attrs)?
        .map(|s| quote! { ::std::option::Option::Some(::std::string::ToString::to_string(#s)) })
        .unwrap_or_else(|| quote! {::std::option::Option::None});

    let s = match &object_args.data {
        Data::Struct(e) => e,
        _ => {
            return Err(
                Error::new_spanned(ident, "MergedObject can only be applied to an struct.").into(),
            )
        }
    };

    let mut types = Vec::new();
    for field in &s.fields {
        types.push(&field.ty);
    }

    let create_merged_obj = {
        let mut obj = quote! { #crate_name::MergedObjectTail };
        for i in 0..types.len() {
            let n = LitInt::new(&format!("{}", i), Span::call_site());
            obj = quote! { #crate_name::MergedObject(&self.#n, #obj) };
        }
        quote! {
            #obj
        }
    };

    let merged_type = {
        let mut obj = quote! { #crate_name::MergedObjectTail };
        for ty in &types {
            obj = quote! { #crate_name::MergedObject::<#ty, #obj> };
        }
        obj
    };

    let expanded = quote! {
        #[allow(clippy::all, clippy::pedantic)]
        impl #crate_name::Type for #ident {
            fn type_name() -> ::std::borrow::Cow<'static, ::std::primitive::str> {
                ::std::borrow::Cow::Borrowed(#gql_typename)
            }

            fn create_type_info(registry: &mut #crate_name::registry::Registry) -> ::std::string::String {
                registry.create_type::<Self, _>(|registry| {
                    #merged_type::create_type_info(registry);

                    let mut fields = ::std::default::Default::default();
                    let mut cache_control = ::std::default::Default::default();

                    if let ::std::option::Option::Some(#crate_name::registry::MetaType::Object(#crate_name::registry::MetaObject {
                        fields: obj_fields,
                        cache_control: obj_cache_control,
                        ..
                    })) = registry.types.get(&*#merged_type::type_name()) {
                        fields = ::std::clone::Clone::clone(obj_fields);
                        cache_control = *obj_cache_control;
                    }

                    #crate_name::registry::MetaType::Object(#crate_name::registry::MetaObject {
                        name: ::std::string::ToString::to_string(#gql_typename),
                        description: #desc,
                        fields,
                        cache_control,
                        extends: #extends,
                        keys: ::std::option::Option::None,
                    })
                })
            }
        }

        #[allow(clippy::all, clippy::pedantic)]
        #[#crate_name::async_trait::async_trait]
        impl #crate_name::resolver_utils::ContainerType for #ident {
            async fn resolve_field(&self, ctx: &#crate_name::Context<'_>) -> #crate_name::ServerResult<::std::option::Option<#crate_name::Value>> {
                #create_merged_obj.resolve_field(ctx).await
            }
        }

        #[allow(clippy::all, clippy::pedantic)]
        #[#crate_name::async_trait::async_trait]
        impl #crate_name::OutputValueType for #ident {
            async fn resolve(&self, ctx: &#crate_name::ContextSelectionSet<'_>, _field: &#crate_name::Positioned<#crate_name::parser::types::Field>) -> #crate_name::ServerResult<#crate_name::Value> {
                #crate_name::resolver_utils::resolve_container(ctx, self).await
            }
        }

        impl #crate_name::ObjectType for #ident {}
    };
    Ok(expanded.into())
}
