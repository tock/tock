#[flux_rs::extern_spec]
#[flux_rs::refined_by(b: bool)]
enum Result<T, E> {
    #[variant({T} -> Result<T, E>[true])]
    Ok(T),
    #[variant({E} -> Result<T, E>[false])]
    Err(E),
}

#[flux_rs::extern_spec]
impl<T, E> Result<T, E> {
    #[sig(fn(&Result<T,E>[@b]) -> bool[b])]
    const fn is_ok(&self) -> bool;

    #[sig(fn(&Result<T,E>[@b]) -> bool[!b])]
    const fn is_err(&self) -> bool;

    #[sig(fn(Result<T, E>[true]) -> T)]
    fn unwrap(self) -> T
    where
        E: core::fmt::Debug;

    #[sig(fn(Result<T, E>[false]) -> E)]
    fn unwrap_err(self) -> E
    where
        T: core::fmt::Debug;
}
