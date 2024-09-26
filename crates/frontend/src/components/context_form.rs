pub mod types;
pub mod utils;
use std::collections::{HashMap, HashSet};

use crate::components::input::{Input, InputType};
use crate::schema::EnumVariants;
use crate::types::Dimension;
use crate::{
    components::dropdown::{Dropdown, DropdownDirection},
    schema::SchemaType,
};
use leptos::*;
use serde_json::Value;
use web_sys::MouseEvent;

use self::types::*;

#[component]
pub fn condition_input(
    disabled: bool,
    resolve_mode: bool,
    allow_remove: bool,
    condition: StoredValue<Condition>,
    input_type: StoredValue<InputType>,
    schema_type: StoredValue<SchemaType>,
    #[prop(into)] on_remove: Callback<String, ()>,
    #[prop(into)] on_value_change: Callback<(usize, Value), ()>,
    #[prop(into)] on_operator_change: Callback<OperatorInput, ()>,
) -> impl IntoView {
    let Condition {
        dimension,
        operator,
        operands,
    } = condition.get_value();

    view! {
        <div class="flex gap-x-6">
            <div class="form-control">
                <label class="label font-mono text-sm">
                    <span class="label-text">Dimension</span>
                </label>
                <input
                    value=dimension.clone()
                    class="input w-full max-w-xs"
                    name="context-dimension-name"
                    disabled=true
                />
            </div>
            <div class="form-control w-20">
                <label class="label font-medium font-mono text-sm">
                    <span class="label-text">Operator</span>
                </label>

                <select
                    disabled=disabled || resolve_mode
                    value=operator.to_string()
                    on:input=move |event| {
                        on_operator_change.call(OperatorInput(event_target_value(&event)));
                    }

                    name="context-dimension-operator"
                    class="select select-bordered w-full max-w-xs text-sm rounded-lg h-10 px-4 appearance-none leading-tight focus:outline-none focus:shadow-outline"
                >
                    <option
                        value="=="
                        selected={ matches!(operator, Operator::Is) } || resolve_mode
                    >
                        "IS"
                    </option>
                    <option value="in" selected=matches!(operator, Operator::In)>
                        "IN"
                    </option>
                    <option value="has" selected=matches!(operator, Operator::Has)>

                        "HAS"
                    </option>
                    <option value="<=" selected=matches!(operator, Operator::Between)>

                        "BETWEEN (inclusive)"
                    </option>
                </select>

            </div>
            <div class="form-control">
                <label class="label font-mono text-sm">
                    <span class="label-text">Value</span>
                </label>
                <div class="flex gap-x-6 items-center">

                    {operands
                        .0
                        .clone()
                        .into_iter()
                        .enumerate()
                        .map(|(idx, operand): (usize, Operand)| {
                            match operand {
                                Operand::Dimension(_) => view! {}.into_view(),
                                Operand::Value(v) => {
                                    view! {
                                        <Input
                                            value=v
                                            schema_type=schema_type.get_value()
                                            on_change=move |value: Value| {
                                                on_value_change.call((idx, value));
                                            }

                                            r#type=input_type.get_value()
                                            disabled=disabled
                                            id=format!(
                                                "{}-{}",
                                                condition
                                                    .with_value(|v| format!("{}-{}", v.dimension, v.operator)),
                                                idx,
                                            )

                                            class=""
                                            name=""
                                            operator=Some(condition.with_value(|v| v.operator.clone()))
                                        />
                                    }
                                        .into_view()
                                }
                            }
                        })
                        .collect_view()} <Show when=move || allow_remove>
                        <button
                            class="btn btn-ghost btn-circle btn-sm mt-1"
                            disabled=disabled
                            on:click=move |_| {
                                on_remove.call(condition.with_value(|v| v.dimension.clone()));
                            }
                        >

                            <i class="ri-delete-bin-2-line text-xl text-2xl font-bold"></i>
                        </button>
                    </Show>
                </div>
            </div>
        </div>
    }
}

#[component]
pub fn context_form<NF>(
    handle_change: NF,
    dimensions: Vec<Dimension>,
    #[prop(default = false)] is_standalone: bool,
    context: Conditions,
    #[prop(default = String::new())] heading_sub_text: String,
    #[prop(default = false)] disabled: bool,
    #[prop(default = DropdownDirection::Right)] dropdown_direction: DropdownDirection,
    #[prop(default = false)] resolve_mode: bool,
) -> impl IntoView
where
    NF: Fn(Conditions) + 'static,
{
    let dimension_map = store_value(
        dimensions
            .iter()
            .map(|v| (v.dimension.clone(), v.clone()))
            .collect::<HashMap<String, Dimension>>(),
    );
    let (used_dimensions, set_used_dimensions) = create_signal(
        context
            .iter()
            .map(|condition| condition.dimension.clone())
            .collect::<HashSet<String>>(),
    );
    let (context, set_context) = create_signal(context.clone());

    let dimensions = StoredValue::new(dimensions);
    let mandatory_dimensions = StoredValue::new(
        dimensions
            .get_value()
            .into_iter()
            .filter_map(|dim| {
                if dim.mandatory {
                    Some(dim.dimension)
                } else {
                    None
                }
            })
            .collect::<HashSet<String>>(),
    );

    let last_idx = create_memo(move |_| context.get().len().max(1) - 1);

    let on_click = move |event: MouseEvent| {
        event.prevent_default();
        logging::log!("Context form submit");
        //TODO: submit logic for this
    };

    create_effect(move |_| {
        let f_context = context.get(); // context will now be a Value
        logging::log!("Context form effect {:?}", f_context);
        handle_change(f_context.clone()); // handle_change now expects Value
    });

    let on_select_dimension = Callback::new(move |selected_dimension: Dimension| {
        let dimension_name = selected_dimension.dimension;
        let r#type = SchemaType::try_from(selected_dimension.schema).unwrap();

        set_used_dimensions.update(|value: &mut HashSet<String>| {
            value.insert(dimension_name.clone());
        });
        set_context.update(|value| {
            value.push(
                Condition::try_from((Operator::Is, dimension_name, r#type)).unwrap(),
            )
        });
    });

    let on_operator_change = Callback::new(
        move |(idx, d_name, d_type, op_input): (
            usize,
            String,
            SchemaType,
            OperatorInput,
        )| {
            set_context.update(|v| {
                if idx < v.len() {
                    let operator: Operator = op_input.into();
                    v[idx].operator = operator.clone();
                    v[idx].operands = Operands::try_from((operator, d_name, d_type))
                        .unwrap_or(Operands(vec![]))
                }
            })
        },
    );

    let on_value_change =
        Callback::new(move |(idx, operand_idx, value): (usize, usize, Value)| {
            set_context.update(|v| {
                if idx < v.len() {
                    let operands = &(v[idx].operands);
                    if operand_idx < operands.len()
                        && matches!(operands[operand_idx], Operand::Value(_))
                    {
                        v[idx].operands[operand_idx] = value.into();
                    }
                }
            })
        });

    let on_remove = Callback::new(move |(idx, d_name): (usize, String)| {
        set_used_dimensions.update(|value| {
            value.remove(&d_name);
        });
        set_context.update(|v| {
            v.remove(idx);
        });
    });

    view! {
        <div class="form-control w-full">
            <div class="gap-1">
                <label class="label flex-col justify-center items-start">
                    <span class="label-text font-semibold text-base">Context</span>
                    <span class="label-text text-slate-400">{heading_sub_text}</span>
                </label>
            </div>
            <div class="card w-full bg-slate-50">
                <div class="card-body">
                    <Show when=move || context.get().is_empty()>
                        <div class="flex justify-center">
                            <Dropdown
                                dropdown_width="w-80"
                                dropdown_icon="ri-add-line".to_string()
                                dropdown_text="Add Context".to_string()
                                dropdown_direction
                                dropdown_options=dimensions.get_value()
                                disabled=disabled
                                on_select=on_select_dimension
                            />
                        </div>
                    </Show>
                    <For
                        each=move || {
                            context
                                .get()
                                .0
                                .into_iter()
                                .enumerate()
                                .collect::<Vec<(usize, Condition)>>()
                        }

                        key=|(idx, condition)| {
                            format!("{}-{}-{}", condition.dimension, idx, condition.operator)
                        }

                        children=move |(idx, condition)| {
                            let schema = dimension_map
                                .with_value(|v| {
                                    v.get(&condition.dimension).unwrap().schema.clone()
                                });
                            let schema_type = store_value(
                                SchemaType::try_from(schema.clone()).unwrap(),
                            );
                            let enum_variants = EnumVariants::try_from(schema);
                            let allow_remove = !disabled
                                && !mandatory_dimensions.get_value().contains(&condition.dimension);
                            let input_type = store_value(
                                InputType::from((
                                    schema_type.get_value(),
                                    enum_variants.unwrap(),
                                    condition.operator.clone(),
                                )),
                            );
                            let condition = store_value(condition);

                            let on_remove = move |d_name| on_remove.call((idx, d_name));
                            let on_value_change = move |(operand_idx, value)| {
                                on_value_change.call((idx, operand_idx, value))
                            };
                            let on_operator_change = move |operator| {
                                on_operator_change
                                    .call((
                                        idx,
                                        condition.with_value(|v| v.dimension.clone()),
                                        schema_type.get_value(),
                                        operator,
                                    ))
                            };
                            view! {
                                // TODO: get rid of unwraps here

                                <ConditionInput
                                    disabled
                                    resolve_mode
                                    allow_remove
                                    condition
                                    input_type
                                    schema_type
                                    on_remove
                                    on_value_change
                                    on_operator_change
                                />
                                {move || {
                                    if last_idx.get() != idx {
                                        view! {
                                            <div class="my-3 ml-5 ml-6 ml-7">
                                                <span class="font-mono text-xs">"&&"</span>
                                            </div>
                                        }
                                            .into_view()
                                    } else {
                                        view! {}.into_view()
                                    }
                                }}
                            }
                        }
                    />

                    <Show when=move || { !context.get().is_empty() && !disabled }>
                        <div class="mt-4">

                            {move || {
                                let dimensions = dimensions
                                    .get_value()
                                    .into_iter()
                                    .filter(|dimension| {
                                        !used_dimensions.get().contains(&dimension.dimension)
                                    })
                                    .collect::<Vec<Dimension>>();
                                view! {
                                    <Dropdown
                                        dropdown_icon="ri-add-line".to_string()
                                        dropdown_text="Add Context".to_string()
                                        dropdown_options=dimensions
                                        disabled=disabled
                                        dropdown_direction
                                        on_select=on_select_dimension
                                    />
                                }
                            }}

                        </div>
                    </Show>

                </div>
            </div>
        </div>
        <Show when=move || is_standalone>
            <div class="flex justify-end">
                <button class="btn" on:click:undelegated=on_click disabled=disabled>
                    Save
                </button>
            </div>
        </Show>
    }
}
