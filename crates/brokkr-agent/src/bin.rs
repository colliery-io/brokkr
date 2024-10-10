use kube::api::{DynamicObject, GroupVersionKind, PatchParams};
use kube::core::object;
use reqwest::Client;
use kube::ResourceExt;
use kube::Client as K8sClient;
use kube::discovery::{ApiCapabilities, ApiResource, Discovery, Scope};
use kube::api::{Api,Patch};
use std::time::Duration;
use std::collections::BTreeMap;
use tokio::time::sleep;
use reqwest::StatusCode;

use brokkr_utils::config::Settings;
use brokkr_utils::logging::prelude::*;
use brokkr_models::models::agents::Agent;
use brokkr_models::models::deployment_objects::DeploymentObject;

use brokkr_agent::broker;

const STACK_LABEL: &str = "k8s.brokkr.io/stack";
const CHECKSUM_ANNOTATION: &str = "k8s.brokkr.io/deployment-checksum";
const LAST_CONFIG_ANNOTATION: &str = "k8s.brokkr.io/last-config-applied";
const DEPLOYMENT_OBJECT_ID_LABEL: &str = "brokkr.io/deployment-object-id";



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
            let is_deletion_marker = d_o.is_deletion_marker;
            
            if is_deletion_marker {
                // TODO: Implement deletion logic
                info!("Processing deletion object: {}", &d_o.id);
                // get all objects matching stack id
                // run delete operation on them
            } else {
                let mut k8s_objects = vec![];
                let yaml_docs = brokkr_agent::utils::multidoc_deserialize(&d_o.yaml_content)?;


                for yaml_doc in yaml_docs {
                    let mut obj: DynamicObject = serde_yaml::from_value(yaml_doc)?;
                    let mut annotations =  BTreeMap::new();
                    annotations.insert(STACK_LABEL.to_string(), d_o.stack_id.to_string());
                    annotations.insert(CHECKSUM_ANNOTATION.to_string(), d_o.yaml_checksum.to_string());
                    annotations.insert(DEPLOYMENT_OBJECT_ID_LABEL.to_string(), d_o.id.to_string());
                    annotations.insert(LAST_CONFIG_ANNOTATION.to_string(), format!("{:?}",obj));
                    obj.metadata.annotations.insert(annotations);

                    let t= obj.types.clone().unwrap().kind; 

                    // move namesapce and CRDs to the front of objects list for apply
                    if t == "Namespace" || t == "CustomResourceDefinition" {
                        k8s_objects.insert(0, obj);
                    } else{
                    k8s_objects.push(obj);
                    }
                }

                // first pass dry runs
                for k8s_object in &k8s_objects{
                    info!("Processing k8s object: {:?}", k8s_object);
                    let default_namespace = &"default".to_string();
                    let namespace = k8s_object.metadata.namespace.as_ref().or(Some(default_namespace)).unwrap();

                    let gvk = if let Some(tm) = &k8s_object.types{
                        GroupVersionKind::try_from(tm)?
                    } else {
                        error!("Cannot apply object without valid TypeMeta {:?}", k8s_object);
                        break;
                    };
                    let name = k8s_object.name_any();
                    if let Some((ar,caps)) = discovery.resolve_gvk(&gvk) {
                        let api = dynamic_api(ar, caps, k8s_client.clone(), Some(namespace), false);
                        info!("Apply {:?}: \n{:?}", gvk.kind, serde_yaml::to_string(&k8s_object));
                        let data = serde_json::to_value(&k8s_object)?;
                        match api.patch(&name, &ss_dry_run, &Patch::Apply(data)).await {
                            Ok(response) => {
                                info!("Dry run successful for {:?} '{}'", gvk.kind, name);
                            },
                            Err(e) => {
                                error!("Dry run failed for {:?} '{}': {:?}", gvk.kind, name, e);
                                // register failed apply event
                                // exit loop
                            }
                        }            
                    }
                }


                // second pass apply
                
                

                for k8s_object in &k8s_objects{
                    info!("Processing k8s object: {:?}", k8s_object);
                    let default_namespace = &"default".to_string();
                    let namespace = k8s_object.metadata.namespace.as_ref().or(Some(default_namespace)).unwrap();

                    let gvk = if let Some(tm) = &k8s_object.types{
                        GroupVersionKind::try_from(tm)?
                    } else {
                        error!("Cannot apply object without valid TypeMeta {:?}", k8s_object);
                        break;
                    };
                    let name = k8s_object.name_any();
                    if let Some((ar,caps)) = discovery.resolve_gvk(&gvk) {
                        let api = dynamic_api(ar, caps, k8s_client.clone(), Some(namespace), false);
                        info!("Apply {:?}: \n{:?}", gvk.kind, serde_yaml::to_string(&k8s_object));
                        let data = serde_json::to_value(&k8s_object)?;
                        match api.patch(&name, &ssapply, &Patch::Apply(data)).await {
                            Ok(response) => {
                                info!("Dry run successful for {:?} '{}'", gvk.kind, name);
                            },
                            Err(e) => {
                                error!("Dry run failed for {:?} '{}': {:?}", gvk.kind, name, e);
                                // register failed apply event
                                // exit loop
                            }
                        }            
                    }
                }

            }
        }

        // Sleep for a while before the next iteration
        tokio::time::sleep(Duration::from_secs(60)).await;
    }
    Ok(())
}



async fn process_deployment_object(object: DeploymentObject) -> Result<(), Box<dyn std::error::Error>> {
    // TODO: Implement deployment logic
    info!("Processing deployment object: {}", object.id);
    // Add your deployment logic here
    Ok(())
}

fn dynamic_api(
    ar: ApiResource,
    caps: ApiCapabilities,
    client: K8sClient,
    ns: Option<&str>,
    all: bool,
) -> Api<DynamicObject> {
    if caps.scope == Scope::Cluster || all {
        Api::all_with(client, &ar)
    } else if let Some(namespace) = ns {
        Api::namespaced_with(client, namespace, &ar)
    } else {
        Api::default_namespaced_with(client, &ar)
    }
}