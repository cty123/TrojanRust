use async_trait::async_trait;

#[async_trait]
pub trait Handler {
    async fn handle(&mut self) -> Result<(), String>;
}