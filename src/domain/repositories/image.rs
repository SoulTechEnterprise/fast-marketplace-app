use async_trait::async_trait;

#[async_trait]
pub trait ImageRepository: Sync + Send {
    async fn add(&self, body: Vec<String>) -> Vec<String>;
    async fn remove(&self) -> ();
}
