use crate::CompressedFile;
use std::pin::Pin;
use tokio::net::TcpStream;

pub struct Context {
    pub stream: Option<TcpStream>,
    pub files: &'static [CompressedFile],
}

#[derive(Debug)]
pub enum ContextError {
    AlreadyTaken,
}

pub type Fut = Pin<Box<dyn Future<Output = ()> + Send>>;

pub struct Stream(TcpStream);
pub struct Files(pub &'static [CompressedFile]);
pub struct File(pub &'static CompressedFile);

pub trait FromContext: Sized + 'static {
    fn from_context(context: &mut Context) -> Result<Self, ContextError>;
}

impl FromContext for Stream {
    fn from_context(context: &mut Context) -> Result<Self, ContextError> {
        if let Some(stream) = context.stream.take() {
            return Ok(Stream(stream));
        }

        Err(ContextError::AlreadyTaken)
    }
}

impl FromContext for Files {
    fn from_context(context: &mut Context) -> Result<Self, ContextError> {
        Ok(Files(context.files))
    }
}

pub trait Handler<T> {
    fn call(self, context: Context) -> Result<Fut, ContextError>;
}

impl<T, F, R> Handler<T> for F
where
    T: FromContext,
    R: Future<Output = ()> + Send + 'static,
    F: Fn(T) -> R,
{
    fn call(self, mut context: Context) -> Result<Fut, ContextError> {
        Ok(Box::pin((self)(T::from_context(&mut context)?)))
    }
}

impl<T1, T2, F, R> Handler<(T1, T2)> for F
where
    T1: FromContext,
    T2: FromContext,
    R: Future<Output = ()> + Send + 'static,
    F: Fn(T1, T2) -> R,
{
    fn call(self, mut context: Context) -> Result<Fut, ContextError> {
        Ok(Box::pin((self)(
            T1::from_context(&mut context)?,
            T2::from_context(&mut context)?,
        )))
    }
}
