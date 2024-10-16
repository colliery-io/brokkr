use brokkr_agent::broker::{send_failure_event, send_success_event};
use kube::api::PatchParams;
use reqwest::Client;
use kube::Client as K8sClient;
use kube::discovery::Discovery;

use std::time::Duration;

use brokkr_utils::config::Settings;
use brokkr_utils::logging::prelude::*;
use brokkr_agent::k8s;

use brokkr_agent::broker;
use crate::k8s::objects::STACK_LABEL;




#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Settings::new(None).expect("Failed to load configuration");
    brokkr_utils::logging::init(&config.log.level).expect("Failed to initialize logger");
    info!("Starting Brokkr Agent");

    
    info!("Waiting for broker to be ready");
    broker::wait_for_broker_ready(&config).await;
    info!("Broker is ready");

    info!("Verifying agent PAK");
    broker::verify_agent_pak(&config).await?;
    info!("Agent PAK verified successfully");

    let client = Client::new();
    info!("HTTP client created");

    info!("Fetching agent details");
    let agent = broker::fetch_agent_details(&config, &client).await?;
    info!("Agent details fetched successfully for agent: {}", agent.name);

    let ssapply = PatchParams::apply(format!("brokkr-agent::{}",agent.id.to_string()).as_str());
    let ss_dry_run = PatchParams::apply(format!("brokkr-agent::{}",agent.id.to_string()).as_str()).dry_run();
    info!("PatchParams created for apply and dry-run");

    info!("Initializing Kubernetes client");
    let k8s_client = K8sClient::try_default().await?;
    let discovery = Discovery::new(k8s_client.clone()).run().await?;
    info!("Kubernetes client initialized successfully");

    info!("Entering main loop");
    loop {
        info!("Fetching and processing deployment objects");
        let deployment_objects = match broker::fetch_and_process_deployment_objects(&config, &client, &agent).await {
            Ok(objects) => {
                info!("Successfully fetched and processed {} deployment objects", objects.len());
                objects
            },
            Err(e) => {
                error!("Error fetching or processing deployment objects: {:?}", e);
                Vec::new()
            }
        };

        for d_o in deployment_objects {
            let d_o_id = d_o.id.clone();
            let is_deletion_marker = d_o.is_deletion_marker;

            info!("Processing deployment object: {}", d_o_id);
            let current_deployment_state = k8s::api::get_all_objects_by_annotation(&k8s_client, &discovery, STACK_LABEL, d_o.stack_id.to_string().as_str()).await?;
            
            if is_deletion_marker {
                info!("Processing deletion object: {}", &d_o.id);
                match k8s::api::delete_k8s_objects(&current_deployment_state, &discovery, k8s_client.clone()).await {
                    Ok(_) => {
                        let success_message = format!("Successfully deleted K8s objects for stack {}", &d_o.stack_id);
                        info!("{}", success_message);
                        send_success_event(&config, &client, &agent, d_o_id, Some(success_message)).await?;
                    },
                    Err(e) => {
                        let error_message = format!("Failed to delete K8s objects: {:?}", e);
                        error!("{}", error_message);
                        send_failure_event(&config, &client, &agent, d_o_id, error_message).await?;
                    }
                };
             } else {
                info!("Creating K8s objects for deployment object: {}", d_o_id);
                let k8s_objects = match k8s::objects::create_k8s_objects(d_o) {
                    Ok(objects) => objects,
                    Err(e) => {
                        let error_message = format!("Failed to create K8s objects: {:?}", e);
                        error!("{}", error_message);
                        send_failure_event(&config, &client, &agent, d_o_id, error_message).await?;
                        continue;
                    }
                };
    
                info!("Performing dry run for deployment object: {}", d_o_id);
                if let Err(e) = k8s::api::apply_k8s_objects(&k8s_objects, &discovery, k8s_client.clone(), &ss_dry_run).await {
                    let error_message = format!("Dry run failed: {}", e);
                    error!("{}", error_message);
                    send_failure_event(&config, &client, &agent, d_o_id, error_message).await?;
                    continue;
                }

                info!("Applying K8s objects for deployment object: {}", d_o_id);
                if let Err(e) = k8s::api::apply_k8s_objects(&k8s_objects, &discovery, k8s_client.clone(), &ssapply).await {
                    let apply_error = format!("Apply failed: {:?}", e);
                    error!("{}", apply_error);
                    let mut error_messages = vec![apply_error];

                    info!("Attempting to delete recently applied objects");
                    if let Err(delete_err) = k8s::api::delete_k8s_objects(&k8s_objects, &discovery, k8s_client.clone()).await {
                        error!("Failed to delete recently applied objects: {:?}", delete_err);
                        error_messages.push(format!("Failed to delete recently applied objects: {}", delete_err));
                    }

                    info!("Attempting to re-apply previous state");
                    if let Err(reapply_err) = k8s::api::apply_k8s_objects(&current_deployment_state, &discovery, k8s_client.clone(), &ssapply).await {
                        error!("Failed to re-apply previous state: {:?}", reapply_err);
                        error_messages.push(format!("Failed to re-apply previous state: {}", reapply_err));
                    }

                    let combined_error_message = error_messages.join("; ");
                    send_failure_event(&config, &client, &agent, d_o_id, combined_error_message).await?;
                    continue;
                }
                
                info!("Successfully applied K8s objects for deployment object {}", d_o_id);
                send_success_event(&config, &client, &agent, d_o_id, None).await?;
            }
        }

        info!("Sleeping for {} seconds before next iteration", config.agent.polling_interval);
        tokio::time::sleep(Duration::from_secs(config.agent.polling_interval)).await;
    }
}




