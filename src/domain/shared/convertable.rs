/// Trait para conversão entre diferentes representações de uma entidade
pub trait Convertable<T> {
    /// Converte a entidade para o tipo T
    fn to(&self) -> T;

    /// Cria uma entidade a partir do tipo T
    fn from(source: T) -> Self;
}
