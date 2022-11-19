use futures::{StreamExt, TryStreamExt};
use k8s_openapi::api::core::v1::Pod;
use tracing::*;

use kube::{
    api::{Api, AttachParams, DeleteParams, ListParams, PostParams, ResourceExt, WatchEvent},
    Client,
};
use tokio::io::AsyncWriteExt;

pub fn default_kube_client() -> kube::Client {
    // TODO: build client config manually to support non-standard kube configs
    Client::try_default().await?
}

// Returns the pod specified by the user
pub fn get_pod(client: kube::Client, name: String, namespace: String) -> Result<Api<Pod>> {
    let pods: Api<Pod> = Api::namespaced(client, namespace.as_str());
    let pod = pods.get(name.as_str()).await?;
}

// Copy the files from the container in the pod to the local filesystem
pub fn copy(client: kube::Client, src: String, dest: String) -> Result<()> {
    Api::

    Ok(())
}
