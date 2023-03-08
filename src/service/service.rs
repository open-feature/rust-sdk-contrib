
pub trait Service {
    fn Resolve<T>() -> anyhow::Result<T>;
}