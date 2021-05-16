use crate::protocol::common::handler::{Handler};

pub struct InboundHandler {
    handler: Box<dyn Handler>
}

pub struct OutboundHandler {
    handler: Box<dyn Handler>
}

impl InboundHandler {
    pub fn new(base_handler: Box<dyn Handler>) -> Box<InboundHandler> {
        return Box::new(InboundHandler {
            handler: base_handler
        })
    }

    pub async fn handle_inbound(&mut self) -> Result<(), String> {
        return match self.handler.handle().await {
            Ok(()) => Ok(()),
            Err(e) => Err(e)
        }
    }
}

impl OutboundHandler {
    pub fn new(base_handler: Box<dyn Handler>) -> Box<InboundHandler> {
        return Box::new(InboundHandler {
            handler: base_handler
        })
    }
}