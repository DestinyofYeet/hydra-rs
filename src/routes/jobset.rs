use chrono::{DateTime, Utc};
use leptos::{prelude::*, task::spawn_local};
use leptos_router::hooks::use_params_map;

use crate::{
    models::{Jobset, JobsetState},
    routes::jobset,
};

#[server]
pub async fn get_jobsets(id: String) -> Result<Vec<Jobset>, ServerFnError> {
    use crate::state::State;
    use axum::http::StatusCode;
    use leptos_axum::ResponseOptions;
    use std::sync::Arc;
    use tracing::error;

    let state: Arc<State> = expect_context();
    let response_opts: ResponseOptions = expect_context();

    let number = id.parse::<i32>();

    if number.is_err() {
        response_opts.set_status(StatusCode::BAD_REQUEST);
        error!("Invalid jobset given");
        return Err(ServerFnError::new("Failed to find project!"));
    }

    let number = number.unwrap();

    let jobsets = state.coordinator.lock().await.get_jobsets(number).await;

    if jobsets.is_err() {
        error!(
            "Failed to fetch jobsets: {}",
            jobsets.err().unwrap().to_string()
        );
        return Err(ServerFnError::new("Failed to fetch jobsets"));
    }

    let jobsets = jobsets.unwrap();

    Ok(jobsets)
}

#[server]
pub async fn get_jobset(id: String) -> Result<Option<Jobset>, ServerFnError> {
    use crate::state::State;
    use axum::http::StatusCode;
    use leptos_axum::ResponseOptions;
    use std::sync::Arc;
    use tracing::error;

    let state: Arc<State> = expect_context();
    let response_opts: ResponseOptions = expect_context();

    let jobset_id = id.parse::<i32>();

    if jobset_id.is_err() {
        response_opts.set_status(StatusCode::BAD_REQUEST);
        error!("Invalid jobset given");
        return Err(ServerFnError::new("Failed to find jobset!"));
    }

    let jobset_id = jobset_id.unwrap();

    let jobset = state.coordinator.lock().await.get_jobset(jobset_id).await;

    if jobset.is_err() {
        error!("Failed to fetch jobset: {}", jobset.err().unwrap());
        return Err(ServerFnError::new("Failed to fetch jobset!"));
    }

    Ok(jobset.unwrap())
}

#[server]
pub async fn trigger_jobset(project_id: String, jobset_id: String) -> Result<(), ServerFnError> {
    use crate::state::State;
    use axum::http::StatusCode;
    use leptos_axum::{redirect, ResponseOptions};
    use std::sync::Arc;
    use tracing::error;
    use tracing::info;

    let jobset = get_jobset(jobset_id.clone()).await?;
    let response_opts: ResponseOptions = expect_context();

    if jobset.is_none() {
        response_opts.set_status(StatusCode::BAD_REQUEST);
        return Err(ServerFnError::new("Failed to find jobset!"));
    }

    let mut jobset = jobset.unwrap();

    let state: Arc<State> = expect_context();

    info!("Triggered jobset: {}", jobset_id);

    let result = state
        .coordinator
        .lock()
        .await
        .schedule_jobset(&mut jobset)
        .await;

    if result.is_err() {
        let err = result.err().unwrap().to_string();
        error!("Failed to schedule jobset: {}", err);
        return Err(ServerFnError::new(err));
    }

    Ok(())
}

#[component]
pub fn Jobset() -> impl IntoView {
    let params = use_params_map();

    let project_id = params.read_untracked().get("proj-id").unwrap_or_default();
    let jobset_id = params.read_untracked().get("jobset-id").unwrap_or_default();

    let (input, _set_input) = signal(jobset_id.clone());

    let jobset_data = Resource::new(
        move || (input.get()),
        |input| async move { get_jobset(input).await },
    );

    let trigger_jobset_action = ServerAction::<TriggerJobset>::new();

    Effect::new(move |_| {
        if let Some(Ok(_)) = trigger_jobset_action.value().get() {
            jobset_data.refetch();
        }
    });

    view! {
        <Suspense fallback=move || view! {<p>"Loading jobset data..."</p>}>
            {move || {
                let jobset = jobset_data.get();

                if jobset.is_none() {
                    return view! {<p>"Error: Failed to load jobset!"</p>}.into_any();
                }

                let jobset = jobset.unwrap();

                if jobset.is_err() {
                    let e = jobset.err().unwrap();
                    let msg = match e {
                        ServerFnError::ServerError(msg) => msg,
                        _ => e.to_string(),
                    };
                    return view! {<p class="error">"Error: Failed to load jobset: "{msg}</p>}.into_any();
                }

                let jobset = jobset.unwrap();

                if jobset.is_none() {
                    return view!{<p>"Error: Failed to find jobset!"</p>}.into_any();
                }

                let jobset = jobset.unwrap();

                view! {
                    <div class="viewjobset">
                        <div class="action">
                            <div class="dropdown">
                                <span>"Actions"</span>
                                <div class="dropdown_content">
                                    <div class="dropdown_group">
                                        <div class="generic_input_form">
                                            <ActionForm action=trigger_jobset_action>
                                                <div class="inputs">
                                                    <input type="hidden" name="project_id" value=jobset.project_id.unwrap().to_string()/>
                                                    <input type="hidden" name="jobset_id" value=jobset.id.unwrap().to_string()/>
                                                    <input type="submit" value="Trigger jobset"/>
                                                </div>
                                            </ActionForm>
                                        </div>
                                    </div>
                                </div>
                            </div>
                        </div>
                        <div class="jobset_trigger_result">
                            {move || {
                                match trigger_jobset_action.value().get() {
                                    Some(Err(e)) => {
                                        let msg = match e {
                                            ServerFnError::ServerError(msg) => msg,
                                            _ => e.to_string(),
                                        };

                                        view! {
                                            <p class="failed">"Failed to trigger jobset: "{msg}</p>
                                        }.into_any()
                                    },

                                    None => {
                                        view! {
                                        }.into_any()
                                    }

                                    _ => {
                                       view! {
                                            <p class="success">"Successfully triggered jobset"</p>
                                       }.into_any()
                                    }
                                }
                            }}
                        </div>
                        <div class="statistics">
                            {mk_jobset_entry("Name: ", jobset.name)}
                            {mk_jobset_entry("Description: ", jobset.description)}
                            {mk_jobset_entry("Flake URI: ", jobset.flake)}
                            {mk_jobset_entry("Last checked: ", convert_date_to_string(jobset.last_checked))}
                            {mk_jobset_entry("Last evaluated: ", convert_date_to_string(jobset.last_evaluated))}
                            {mk_jobset_entry("Evaluation took: ", format!("{}", jobset.evaluation_took.unwrap_or(-1)))}
                            {mk_jobset_entry("State: ", jobset.state.unwrap_or(JobsetState::UNKNOWN).to_string())}
                        </div>
                    </div>
                }.into_any()
            }}
        </Suspense>
    }
}

fn convert_date_to_string(date: Option<DateTime<Utc>>) -> String {
    match date {
        None => "never".to_string(),
        Some(value) => value.to_rfc3339(),
    }
}

fn mk_jobset_entry(key: &str, value: String) -> impl IntoView {
    view! {
        <div class="key">
            <p>{key.to_string()}</p>
        </div>
        <div class="value">
            <p>{value}</p>
        </div>
    }
}
