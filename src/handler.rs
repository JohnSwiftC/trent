use crate::CompressedFile;
use tokio::net::TcpStream;

pub struct Context {
    stream: TcpStream,
    files: &'static [CompressedFile],
}

// The *extracted* types you want handlers to receive:
pub struct Stream<'a>(pub &'a mut TcpStream);
pub struct Files(pub &'static [CompressedFile]);

// Marker extractor types (these are what implement FromContext)
pub struct StreamArg;
pub struct FilesArg;

pub trait FromContext<'a> {
    type Output;
    fn from_context(context: &'a Context) -> Self::Output;
}

pub trait FromContextBorrowed<'a> {
    type Output;
    fn from_context(context: &'a mut Context) -> Self::Output;
}

pub struct Mut<E>(std::marker::PhantomData<E>);

impl<'a> FromContextBorrowed<'a> for StreamArg {
    type Output = Stream<'a>;

    fn from_context(context: &'a mut Context) -> Self::Output {
        Stream(&mut context.stream)
    }
}

impl<'a> FromContext<'a> for FilesArg {
    type Output = Files;

    fn from_context(context: &'a Context) -> Self::Output {
        Files(context.files)
    }
}

pub trait Handler<E> {
    fn call(self, state: Context);
}

impl<E, F> Handler<E> for F
where
    for<'a> E: FromContext<'a>,
    for<'a> F: Fn(<E as FromContext<'a>>::Output),
{
    fn call(self, context: Context) {
        let arg = <E as FromContext<'_>>::from_context(&context);
        self(arg);
    }
}

impl<E, F> Handler<Mut<E>> for F
where
    for<'a> E: FromContextBorrowed<'a>,
    for<'a> F: Fn(<E as FromContextBorrowed<'a>>::Output),
{
    fn call(self, mut context: Context) {
        let arg = <E as FromContextBorrowed<'_>>::from_context(&mut context);
        self(arg);
    }
}

impl<E1, E2, F, O1> Handler<(E1, E2)> for F
where
    for<'a> E1: FromContext<'a, Output = O1>,
    for<'a> E2: FromContextBorrowed<'a>,
    for<'a> F: Fn(O1, <E2 as FromContextBorrowed<'a>>::Output),
    O1: 'static,
{
    fn call(self, mut context: Context) {
        let arg1 = <E1 as FromContext<'_>>::from_context(&context);
        let arg2 = <E2 as FromContextBorrowed<'_>>::from_context(&mut context);
        self(arg1, arg2);
    }
}
