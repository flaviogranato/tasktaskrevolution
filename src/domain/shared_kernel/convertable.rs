pub trait Convertable<T> {
    fn from(source: T) -> Self;
    fn to(self) -> T;
}
