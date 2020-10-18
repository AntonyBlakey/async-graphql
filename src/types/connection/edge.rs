use std::borrow::Cow;

use indexmap::map::IndexMap;

use crate::connection::EmptyFields;
use crate::parser::types::Field;
use crate::resolver_utils::{resolve_container, ContainerType};
use crate::types::connection::CursorType;
use crate::{
    registry, Context, ContextSelectionSet, ObjectType, OutputValueType, Positioned, ServerResult,
    Type, Value,
};

/// The edge type output by the data source
pub struct Edge<C, T, E> {
    pub(crate) cursor: C,
    pub(crate) node: T,
    pub(crate) additional_fields: E,
}

impl<C, T, E> Edge<C, T, E> {
    /// Create a new edge, it can have some additional fields.
    pub fn with_additional_fields(cursor: C, node: T, additional_fields: E) -> Self {
        Self {
            cursor,
            additional_fields,
            node,
        }
    }
}

impl<C: CursorType, T> Edge<C, T, EmptyFields> {
    /// Create a new edge.
    pub fn new(cursor: C, node: T) -> Self {
        Self {
            cursor,
            node,
            additional_fields: EmptyFields,
        }
    }
}

impl<C, T, E> Type for Edge<C, T, E>
where
    C: CursorType,
    T: OutputValueType + Send + Sync,
    E: ObjectType + Sync + Send,
{
    fn type_name() -> Cow<'static, str> {
        Cow::Owned(format!("{}Edge", T::type_name()))
    }

    fn create_type_info(registry: &mut registry::Registry) -> String {
        registry.create_type::<Self, _>(|registry| {
            E::create_type_info(registry);
            let additional_fields =
                if let Some(registry::MetaType::Object(registry::MetaObject { fields, .. })) =
                    registry.types.get(E::type_name().as_ref())
                {
                    fields.clone()
                } else {
                    unreachable!()
                };

            registry::MetaType::Object(registry::MetaObject {
                name: Self::type_name().to_string(),
                description: Some("An edge in a connection.".to_string()),
                fields: {
                    let mut fields = IndexMap::new();

                    fields.insert(
                        "node".to_string(),
                        registry::MetaField {
                            name: "node".to_string(),
                            description: Some("The item at the end of the edge".to_string()),
                            args: Default::default(),
                            ty: T::create_type_info(registry),
                            deprecation: None,
                            cache_control: Default::default(),
                            external: false,
                            requires: None,
                            provides: None,
                        },
                    );

                    fields.insert(
                        "cursor".to_string(),
                        registry::MetaField {
                            name: "cursor".to_string(),
                            description: Some("A cursor for use in pagination".to_string()),
                            args: Default::default(),
                            ty: String::create_type_info(registry),
                            deprecation: None,
                            cache_control: Default::default(),
                            external: false,
                            requires: None,
                            provides: None,
                        },
                    );

                    fields.extend(additional_fields);
                    fields
                },
                cache_control: Default::default(),
                extends: false,
                keys: None,
            })
        })
    }
}

#[async_trait::async_trait]
impl<C, T, E> ContainerType for Edge<C, T, E>
where
    C: CursorType + Send + Sync,
    T: OutputValueType + Send + Sync,
    E: ObjectType + Sync + Send,
{
    async fn resolve_field(&self, ctx: &Context<'_>) -> ServerResult<Option<Value>> {
        if ctx.item.node.name.node == "node" {
            let ctx_obj = ctx.with_selection_set(&ctx.item.node.selection_set);
            return OutputValueType::resolve(&self.node, &ctx_obj, ctx.item)
                .await
                .map(Some);
        } else if ctx.item.node.name.node == "cursor" {
            return Ok(Some(Value::String(self.cursor.encode_cursor())));
        }

        self.additional_fields.resolve_field(ctx).await
    }
}

#[async_trait::async_trait]
impl<C, T, E> OutputValueType for Edge<C, T, E>
where
    C: CursorType + Send + Sync,
    T: OutputValueType + Send + Sync,
    E: ObjectType + Sync + Send,
{
    async fn resolve(
        &self,
        ctx: &ContextSelectionSet<'_>,
        _field: &Positioned<Field>,
    ) -> ServerResult<Value> {
        resolve_container(ctx, self).await
    }
}

impl<C, T, E> ObjectType for Edge<C, T, E>
where
    C: CursorType + Send + Sync,
    T: OutputValueType + Send + Sync,
    E: ObjectType + Sync + Send,
{
}
