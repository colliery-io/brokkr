// use kube::{Client, api::{Api, PostParams}};
// use k8s_openapi::api::core::v1::Pod;
// use brokkr_models::models::deployment_objects::DeploymentObject;

// pub struct K8sClient {
//     client: Client,
// }

// impl K8sClient {
//     pub async fn new(kubeconfig_path: Option<String>) -> Self {
//         let client = match kubeconfig_path {
//             Some(path) => Client::try_from(kube::Config::from_kubeconfig(&path).await.unwrap()).unwrap(),
//             None => Client::try_default().await.unwrap(),
//         };
//         Self { client }
//     }

//     pub async fn apply_deployment_object(&self, deployment_object: &DeploymentObject) -> Result<(), kube::Error> {
//         let pods: Api<Pod> = Api::default_namespaced(self.client.clone());
//         let pod: Pod = serde_yaml::from_str(&deployment_object.yaml_content).unwrap();
//         pods.create(&PostParams::default(), &pod).await?;
//         Ok(())
//     }
// }