use std::collections::HashMap;
use std::rc::Rc;

use leptos::logging::*;
use leptos::*;
use serde_json::{json, Map, Value};
use web_sys::SubmitEvent;

use crate::components::table::types::TableSettings;
use crate::components::table::{table::Table, types::Column};
use crate::pages::Dimensions::helper::fetch_dimensions;

#[derive(Clone, Debug, Default)]
pub struct RowData {
    pub dimension: String,
    pub priority: String,
    pub type_: String,
    pub pattern: String
}

pub fn custom_formatter(_value: &str, row: &Map<String, Value>) -> View {
  let intermediate_signal = use_context::<RwSignal<Option<RowData>>>().unwrap();
  let row_dimension = row["dimension"].clone().to_string().replace("\"", "");
  let row_priority = row["priority"].clone().to_string().replace("\"", "");

  let schema = row["schema"].clone().to_string();
  let schema_object = serde_json::from_str::<serde_json::Value>(&schema).unwrap();
  
  let row_type = schema_object.get("type").unwrap().to_string();
  let row_pattern = schema_object.get("pattern").unwrap().to_string();

  let edit_click_handler = move |_| {
      let row_data = RowData {
          dimension: row_dimension.clone(),
          priority: row_priority.clone(),
          type_: row_type.clone(),
          pattern: row_pattern.clone(),
      };
      intermediate_signal.set(Some(row_data));
      js_sys::eval("document.getElementById('my_modal_5').showModal();").unwrap();
  };

  let edit_icon: HtmlElement<html::I> = view! { <i class="ri-pencil-line ri-xl text-blue-500" on:click=edit_click_handler></i> };

  view! { <span class="cursor-pointer">{edit_icon}</span> }.into_view()
}

pub async fn create_dimension(
  tenant: String,
  key: String,
  priority: String,
  key_type: String,
  pattern: String,
) -> Result<String, String> {
  let priority: i64 = priority.parse().unwrap();
  let client = reqwest::Client::new();
  let host = "http://localhost:8080";
  let url = format!("{host}/dimension");

  let mut req_body: HashMap<&str, Value> = HashMap::new();
  let mut schema: Map<String, Value> = Map::new();
  
  schema.insert("type".to_string(), Value::String(key_type.replace("\"", "")));
  schema.insert("pattern".to_string(), Value::String(pattern.replace("\"", "")));

  req_body.insert("dimension", Value::String(key));
  req_body.insert("priority", Value::Number(priority.into()));
  req_body.insert("schema", Value::Object(schema));

  let response = client
      .put(url)
      .header("x-tenant", tenant)
      .header("Authorization", "Bearer 12345678")
      .json(&req_body)
      .send()
      .await
      .map_err(|e| e.to_string())?;
  response.text().await.map_err(|e| e.to_string())
}

#[component]
fn ModalComponent(handle_submit: Rc<dyn Fn()>, tenant: ReadSignal<String>) -> impl IntoView {
    view! {
        <div class="pt-4">
            <button class="btn btn-outline btn-primary" onclick="my_modal_5.showModal()">
                Create Dimension
                <i class="ri-edit-2-line ml-2"></i>
            </button>
            <FormComponent handle_submit=handle_submit tenant=tenant/>
        </div>
    }
}

#[component]
fn FormComponent(handle_submit: Rc<dyn Fn()>, tenant: ReadSignal<String>) -> impl IntoView {
    use leptos::html::Input;
    let handle_submit = handle_submit.clone();
    let global_state = use_context::<RwSignal<RowData>>();
    let row_data = global_state.unwrap().get();

    let (dimension, set_dimension) = create_signal(row_data.dimension);
    let (priority, set_priority) = create_signal(row_data.priority);
    let (keytype, set_keytype) = create_signal(row_data.type_);
    let (pattern, set_pattern) = create_signal(row_data.pattern);

    create_effect(move |_| {
        if let Some(row_data) = global_state {
            set_dimension.set(row_data.get().dimension.clone().to_string());
            set_priority.set(row_data.get().priority.clone());
            set_keytype.set(row_data.get().type_.clone().to_string());
            set_pattern.set(row_data.get().pattern.clone());
        }
    });

    let input_element: NodeRef<Input> = create_node_ref();
    let input_element_two: NodeRef<Input> = create_node_ref();
    let input_element_three: NodeRef<Input> = create_node_ref();
    let input_element_four: NodeRef<Input> = create_node_ref();

    let on_submit = {
        let handle_submit = handle_submit.clone();
        move |ev: SubmitEvent| {
            ev.prevent_default();

            let value1 = input_element.get().expect("<input> to exist").value();
            let value2 = input_element_two.get().expect("<input> to exist").value();
            let value3 = input_element_three.get().expect("<input> to exist").value();
            let value4 = input_element_four.get().expect("<input> to exist").value();

            set_dimension.set(value1.clone());
            set_priority.set(value2.clone());
            set_keytype.set(value3.clone());
            set_pattern.set(value4.clone());
            let handle_submit_clone = handle_submit.clone();

            spawn_local({
                let handle_submit = handle_submit_clone;
                async move {
                    let result = create_dimension(
                        tenant.get(),
                        dimension.get(),
                        priority.get(),
                        keytype.get(),
                        pattern.get(),
                    )
                    .await;

                    match result {
                        Ok(_) => {
                            handle_submit();
                        }
                        Err(_) => {
                            // Handle error
                            // Consider logging or displaying the error
                        }
                    }
                }
            });
        }
    };

    view! {
      <dialog id="my_modal_5" class="modal modal-bottom sm:modal-middle">
                <div class="modal-box relative bg-white">
                    <form method="dialog" class="flex justify-end">
                        <button>
                            <i class="ri-close-fill" onclick="my_modal_5.close()"></i>
                        </button>
                    </form>
        <form
            class="form-control w-full space-y-4 bg-white text-gray-700 font-mono"
            on:submit=on_submit
        >
            <div class="form-control">
                <label class="label font-mono">
                    <span class="label-text text-gray-700 font-mono">Dimension</span>
                </label>
                <input
                    type="text"
                    placeholder="Dimension"
                    class="input input-bordered w-full bg-white text-gray-700 shadow-md"
                    value=dimension
                    node_ref=input_element
                />
            </div>
            <div class="form-control">
                <label class="label font-mono">
                    <span class="label-text text-gray-700 font-mono">Priority</span>
                </label>
                <input
                    type="Number"
                    placeholder="Priority"
                    class="input input-bordered w-full bg-white text-gray-700 shadow-md"
                    value=priority
                    node_ref=input_element_two
                />
            </div>
            <div class="form-control">
                <label class="label font-mono">
                    <span class="label-text text-gray-700 font-mono">Type</span>
                </label>
                <input
                    type="text"
                    placeholder="Type"
                    class="input input-bordered w-full bg-white text-gray-700 shadow-md"
                    value=keytype
                    node_ref=input_element_three
                />
            </div>
            <div class="form-control">
                <label class="label font-mono">
                    <span class="label-text text-gray-700 font-mono">Pattern (regex)</span>
                </label>
                <input
                    type="text"
                    placeholder="Pattern"
                    class="input input-bordered w-full bg-white text-gray-700 shadow-md"
                    value=pattern
                    node_ref=input_element_four
                />
            </div>
            <div class="form-control mt-6">
                <button
                    type="submit"
                    class="btn btn-primary shadow-md font-mono"
                    onclick="my_modal_5.close()"
                >
                    Submit
                </button>
            </div>
        </form>
        </div>
      </dialog>
    }
}


#[component]
pub fn Dimensions() -> impl IntoView {
  let tenant_rs = use_context::<ReadSignal<String>>().unwrap();
  let global_state = create_rw_signal(RowData::default());
    provide_context(global_state);

    let intermediate_signal = create_rw_signal(None::<RowData>);

    create_effect(move |_| {
        if let Some(row_data) = intermediate_signal.get() {
            global_state.set(row_data.clone());
        }
    });

    provide_context(intermediate_signal.clone());

    let dimensions = create_blocking_resource(
        move || {},
        |_value| async move {
            match fetch_dimensions().await {
                Ok(data) => data,
                Err(_) => vec![],
            }
        },
    );

    let table_columns = create_memo(move |_| {
        vec![
            Column::default("dimension".to_string()),
            Column::default("priority".to_string()),
            Column::default("schema".to_string()),
            Column::default("created_by".to_string()),
            Column::default("created_at".to_string()),
            Column::new("EDIT".to_string(), None, Some(custom_formatter)),
        ]
    });

    view! {
        <div class="p-8">
            <Suspense fallback=move || view! { <p>"Loading (Suspense Fallback)..."</p> }>
                <div class="pb-4">
                    <div class="stats shadow">
                        <div class="stat">
                            <div class="stat-figure text-primary">
                                <i class="ri-ruler-2-fill text-5xl"></i>
                            </div>
                            <div class="stat-title">Dimensions</div>

                            {move || {
                                let value = dimensions.get();
                                let total_items = match value {
                                    Some(v) => v.len(),
                                    _ => 0,
                                };
                                view! { <div class="stat-value">{total_items}</div> }
                            }}

                        </div>
                    </div>
                    <ModalComponent handle_submit=Rc::new(move || dimensions.refetch()) tenant=tenant_rs/>
                </div>

                <div class="card rounded-xl w-full bg-base-100 shadow">
                    <div class="card-body">
                        <h2 class="card-title">Dimensions</h2>
                        <div>

                            {move || {
                                let value = dimensions.get();
                                let settings = TableSettings {
                                    redirect_prefix: None
                                };
                                match value {
                                    Some(v) => {
                                        let data = v
                                            .iter()
                                            .map(|ele| { json!(ele).as_object().unwrap().clone() })
                                            .collect::<Vec<Map<String, Value>>>()
                                            .to_owned();
                                        view! {
                                            <Table
                                                table_style="abc".to_string()
                                                rows= data
                                                key_column="id".to_string()
                                                columns=table_columns.get()
                                                settings=settings
                                            />
                                        }
                                    }
                                    None => view! { <div>Loading....</div> }.into_view(),
                                }
                            }}

                        </div>
                    </div>
                </div>
            </Suspense>
        </div>
    }
}
