
#[path ="src/providers/flagd/proto/rust"]
pub mod rust;

pub trait Service {
    fn Resolve<T>() -> anyhow::Result<T>;
}

impl DefaultService for Service {
    fn new() -> Self {
        Service {
            
        }
    }
    fn Resolve<T>() -> anyhow::Result<T> {
        Ok(T)
    }
}