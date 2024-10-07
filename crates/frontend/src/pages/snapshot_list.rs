use leptos::*;

use chrono::{DateTime, TimeZone, Utc};
use leptos_router::A;
use serde_json::{json, Map, Value};

use crate::components::skeleton::Skeleton;
use crate::components::stat::Stat;
use crate::components::table::types::TablePaginationProps;
use crate::components::table::{types::Column, Table};
use crate::types::{ListFilters, Snapshot};

use crate::api::fetch_snapshots;

#[component]
pub fn snapshot_list() -> impl IntoView {
    let tenant_rs = use_context::<ReadSignal<String>>().unwrap();

    // Signals for filters
    let (filters, set_filters) = create_signal(ListFilters {
        status: None,
        from_date: Utc.timestamp_opt(0, 0).single(),
        to_date: Utc.timestamp_opt(4130561031, 0).single(),
        page: Some(1),
        count: Some(10), // Limit of 10 items per page
    });

    let table_columns = create_memo(move |_| snapshot_table_columns(tenant_rs.get()));

    let snapshots_resource: Resource<String, Vec<Snapshot>> = create_blocking_resource(
        move || tenant_rs.get(),
        |current_tenant| async move {
            match fetch_snapshots(current_tenant.to_string()).await {
                Ok(response) => response.data,
                Err(_) => vec![],
            }
        },
    );

    let handle_next_click = Callback::new(move |total_pages: i64| {
        set_filters.update(|f| {
            f.page = match f.page {
                Some(p) if p < total_pages => Some(p + 1),
                Some(p) => Some(p),
                None => Some(1),
            }
        });
    });

    let handle_prev_click = Callback::new(move |_| {
        set_filters.update(|f| {
            f.page = match f.page {
                Some(p) if p > 1 => Some(p - 1),
                Some(_) => Some(1),
                None => Some(1),
            }
        });
    });

    view! {
        <div class="p-8">
            <Suspense fallback=move || view! { <Skeleton/> }>
                <div class="pb-4">
                    {move || {
                        let snapshot_res = snapshots_resource.get();
                        let total_items = match snapshot_res {
                            Some(snapshot_resp) => snapshot_resp.len().to_string(),
                            _ => "0".to_string(),
                        };
                        view! {
                            <Stat
                                heading="Snapshots"
                                icon="ri-camera-lens-fill"
                                number=total_items
                            />
                        }
                    }}

                </div>
                <div class="card rounded-xl w-full bg-base-100 shadow">
                    <div class="card-body">
                        <div class="flex justify-between">
                            <h2 class="card-title">Snapshots</h2>
                        </div>
                        <div>
                            {move || {
                                let value = snapshots_resource.get();
                                let filters = filters.get();
                                match value {
                                    Some(v) => {
                                        let page = filters.page.unwrap_or(1);
                                        let count = filters.count.unwrap_or(10);
                                        let total_items = v.len();
                                        let total_pages = (total_items as f64 / count as f64).ceil()
                                            as i64;
                                        let start = ((page - 1) * count as i64) as usize;
                                        let end = std::cmp::min(
                                            start + count as usize,
                                            total_items,
                                        );
                                        let data = v[start..end]
                                            .iter()
                                            .map(|snapshot| {
                                                let mut snapshot_map = json!(snapshot)
                                                    .as_object()
                                                    .unwrap()
                                                    .to_owned();
                                                if let Some(id_value) = snapshot_map.get("id") {
                                                    let id_string = match id_value {
                                                        Value::Number(num) => num.to_string(),
                                                        Value::String(s) => s.clone(),
                                                        _ => "".to_string(),
                                                    };
                                                    snapshot_map
                                                        .insert("id".to_string(), Value::String(id_string));
                                                }
                                                if let Some(created_at_str) = snapshot_map
                                                    .get("created_at")
                                                    .and_then(|v| v.as_str())
                                                {
                                                    if let Ok(created_at_dt) = DateTime::parse_from_rfc3339(
                                                        created_at_str,
                                                    ) {
                                                        snapshot_map
                                                            .insert(
                                                                "created_at".to_string(),
                                                                json!(created_at_dt.format("%Y-%m-%d %H:%M:%S").to_string()),
                                                            );
                                                    }
                                                }
                                                snapshot_map
                                            })
                                            .collect::<Vec<Map<String, Value>>>()
                                            .to_owned();
                                        let pagination_props = TablePaginationProps {
                                            enabled: true,
                                            count,
                                            current_page: page,
                                            total_pages,
                                            on_next: handle_next_click,
                                            on_prev: handle_prev_click,
                                        };
                                        view! {

                                            <Table
                                                cell_class="".to_string()
                                                rows=data
                                                key_column="id".to_string()
                                                columns=table_columns.get()
                                                pagination=pagination_props
                                            />
                                        }
                                    }
                                    None => {
                                        view! {

                                            <div>Loading....</div>
                                        }
                                            .into_view()
                                    }
                                }
                            }}

                        </div>
                    </div>
                </div>
            </Suspense>
        </div>
    }
}

pub fn snapshot_table_columns(tenant: String) -> Vec<Column> {
    vec![
        Column::new(
            "id".to_string(),
            None,
            move |value: &str, _row: &Map<String, Value>| {
                let id = value.to_string();
                let href = format!("/admin/{}/snapshots/{}", tenant.clone(), id);
                view! {
                    <div>
                        <A href=href class="btn-link">{id}</A>
                    </div>
                }
                .into_view()
            },
        ),
        Column::default("created_at".to_string()),
        Column::new(
            "tags".to_string(),
            None,
            |_value: &str, row: &Map<String, Value>| {
                let tags = row.get("tags").and_then(|v| v.as_array());
                match tags {
                    Some(arr) => {
                        let tags_str = arr
                            .iter()
                            .map(|v| v.as_str().unwrap_or(""))
                            .collect::<Vec<&str>>()
                            .join(", ");
                        view! { <span>{tags_str}</span> }
                    }
                    None => view! { <span>"-"</span> },
                }
                .into_view()
            },
        ),
    ]
}
