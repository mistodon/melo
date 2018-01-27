pub trait Trust<T>
{
    fn trust(self) -> T;
}

impl<T> Trust<T> for Option<T>
{
    fn trust(self) -> T
    {
        self.unwrap_or_else(|| unreachable!("trust was broken"))
    }
}

impl<T, E> Trust<T> for Result<T, E>
{
    fn trust(self) -> T
    {
        self.unwrap_or_else(|_| unreachable!("trust was broken"))
    }
}
