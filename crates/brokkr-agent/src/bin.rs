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
    
    broker::wait_for_broker_ready(&config).await;
    broker::verify_agent_pak(&config).await?;

    let client = Client::new();
    let agent = broker::fetch_agent_details(&config, &client).await?;

    let ssapply = PatchParams::apply(format!("brokkr-agent::{}",agent.id.to_string()).as_str());
    let ss_dry_run = PatchParams::apply(format!("brokkr-agent::{}",agent.id.to_string()).as_str()).dry_run();

    let k8s_client = K8sClient::try_default().await?;
    let discovery = Discovery::new(k8s_client.clone()).run().await?;

    loop {


        let deployment_objects = match broker::fetch_and_process_deployment_objects(&config, &client, &agent).await {
            Ok(objects) => {
                info!("Successfully fetched and processed deployment objects");
                objects
            },
            Err(e) => {
                error!("Error fetching or processing deployment objects: {:?}", e);
                // Implement retry logic or additional error handling here if needed
                Vec::new() // Return an empty vector in case of error
            }
        };

        for d_o in deployment_objects {

            let d_o_id = d_o.id.clone();
            let is_deletion_marker = d_o.is_deletion_marker;

            let current_deployment_state = k8s::api::get_all_objects_by_annotation(&k8s_client, &discovery, STACK_LABEL, d_o.stack_id.to_string().as_str()).await?;
            
            if is_deletion_marker {
                // TODO: Implement deletion logic
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

                let k8s_objects = match k8s::objects::create_k8s_objects(d_o) {
                    Ok(objects) => objects,
                    Err(e) => {
                        let error_message = format!("Failed to create K8s objects: {:?}", e);
                        error!("{}", error_message);
                        send_failure_event(&config, &client, &agent, d_o_id, error_message).await?;
                        continue; // Skip to the next deployment object
                    }
                };
    
            // Dry run
            if let Err(e) = k8s::api::apply_k8s_objects(&k8s_objects, &discovery, k8s_client.clone(), &ss_dry_run).await {
                let error_message = format!("Dry run failed: {}", e);
                error!("{}", error_message);
                send_failure_event(&config, &client, &agent, d_o_id, error_message).await?;
                continue;
            }


            // Actual apply
            if let Err(e) = k8s::api::apply_k8s_objects(&k8s_objects, &discovery, k8s_client.clone(), &ssapply).await {
                let apply_error = format!("Apply failed: {:?}", e);
                error!("{}", apply_error);
                let mut error_messages = vec![apply_error];

                // Delete recently applied objects
                if let Err(delete_err) = k8s::api::delete_k8s_objects(&k8s_objects, &discovery, k8s_client.clone()).await {
                    error!("Failed to delete recently applied objects: {:?}", delete_err);
                    error_messages.push(format!("Failed to delete recently applied objects: {}", delete_err));
                }

                // Re-apply the previous state
                if let Err(reapply_err) = k8s::api::apply_k8s_objects(&current_deployment_state, &discovery, k8s_client.clone(), &ssapply).await {
                    error!("Failed to re-apply previous state: {:?}", reapply_err);
                    error_messages.push(format!("Failed to re-apply previous state: {}", reapply_err));
                }

                // Send a single failure event with all error messages combined
                let combined_error_message = error_messages.join("; ");
                send_failure_event(&config, &client, &agent, d_o_id, combined_error_message).await?;
                continue;
            }
                
            // Send success event if we've reached this point without any failures
            send_success_event(&config, &client, &agent, d_o_id, None).await?;
            info!("Successfully applied K8s objects for deployment object {}", d_o_id);

                

            }
        }

        // Sleep for the configured polling interval before the next iteration
        tokio::time::sleep(Duration::from_secs(config.agent.polling_interval)).await;
    }
    Ok(())
}




