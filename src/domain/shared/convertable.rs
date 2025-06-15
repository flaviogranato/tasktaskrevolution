pub trait Convertable<T> {
    fn to(&self) -> T;
    fn from(source: T) -> Self;
}
