use leptos::*;
use serde_json::{Map, Value};

use crate::{
    components::{
        condition_pills::{
            types::{Condition, ConditionOperator},
            Condition as ConditionComponent,
        },
        table::{types::Column, Table},
    },
    types::Context,
};

#[component]
pub fn context_card(
    context: Context,
    overrides: Map<String, Value>,
    #[prop(default = true)] show_actions: bool,
    #[prop(default=Callback::new(|_| {}))] handle_edit: Callback<
        (Context, Map<String, Value>),
        (),
    >,
    #[prop(default=Callback::new(|_| {}))] handle_clone: Callback<
        (Context, Map<String, Value>),
        (),
    >,
    #[prop(default=Callback::new(|_| {}))] handle_delete: Callback<String, ()>,
) -> impl IntoView {
    let conditions: Vec<Condition> = (&context).try_into().unwrap_or(vec![]);
    let override_table_rows = overrides
        .clone()
        .into_iter()
        .map(|(k, v)| {
            let k = Value::String(k.trim_matches('"').to_string());
            let v = Value::String(format!("{}", v).trim_matches('"').to_string());
            Map::from_iter(vec![(String::from("KEY"), k), (String::from("VALUE"), v)])
        })
        .collect::<Vec<Map<String, Value>>>();

    let context_id = StoredValue::new(context.id.clone());
    let context = StoredValue::new(context);
    let overrides = StoredValue::new(overrides);

    let table_columns = vec![
        Column::default("KEY".to_string()),
        Column::default("VALUE".to_string()),
    ];

    let actions_supported = show_actions
        && !conditions
            .iter()
            .any(|condition| condition.left_operand == "variantIds");

    let edit_unsupported = conditions
        .iter()
        .any(|condition| matches!(condition.operator, ConditionOperator::Other(_)));

    view! {
        <div class="rounded-lg shadow bg-base-100 p-6 shadow flex flex-col gap-4">
            <div class="flex justify-between">
                <h3 class="card-title text-base timeline-box text-gray-800 bg-base-100 shadow-md font-mono m-0 w-max">
                    "Condition"
                </h3>
                <Show when=move || actions_supported>
                    <div class="h-fit text-right space-x-4">
                        <Show when=move || !edit_unsupported>
                            <i
                                class="ri-pencil-line ri-lg text-blue-500 cursor-pointer"
                                on:click=move |_| {
                                    handle_edit.call((context.get_value(), overrides.get_value()));
                                }
                            ></i>

                            <i
                                class="ri-file-copy-line ri-lg text-blue-500 cursor-pointer"
                                on:click=move |_| {
                                    handle_clone.call((context.get_value(), overrides.get_value()));
                                }
                            ></i>

                        </Show>
                        <Show when=move || edit_unsupported>
                            <span class="badge badge-warning text-xs ml-2 flex items-center">
                                {"Edit Unsupported"}
                            </span>
                        </Show>
                        <i
                            class="ri-delete-bin-5-line ri-lg text-red-500 cursor-pointer"
                            on:click=move |_| {
                                let context_id = context_id.get_value();
                                handle_delete.call(context_id);
                            }
                        ></i>

                    </div>
                </Show>
                <Show when=move || !actions_supported>
                    <span class="badge badge-warning text-xs ml-2 flex items-center">
                        {"Edit Unsupported"}
                    </span>
                </Show>
            </div>

            <div class="pl-5">
                <ConditionComponent
                    conditions=conditions
                    id=context_id.get_value()
                    class="xl:w-[400px] h-fit"
                />
                <Table
                    cell_class="min-w-48 font-mono".to_string()
                    rows=override_table_rows
                    key_column="id".to_string()
                    columns=table_columns
                />
            </div>
        </div>
    }
}
