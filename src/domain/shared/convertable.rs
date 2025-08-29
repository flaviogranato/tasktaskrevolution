/// A trait for objects that can be converted to and from other types
pub trait Convertible<T> {
    /// Convert self to the target type
    fn to(&self) -> T;

    /// Create self from the source type
    fn from(source: T) -> Self;
}

/// A trait for objects that can be converted to other types
pub trait Into<T> {
    /// Convert self into the target type
    fn into(self) -> T;
}

/// A trait for objects that can be created from other types
pub trait From<T> {
    /// Create self from the source type
    fn from(source: T) -> Self;
}

// Implementação padrão para tipos que implementam From
impl<T, U> Into<U> for T
where
    U: From<T>,
{
    fn into(self) -> U {
        U::from(self)
    }
}

// Implementação padrão para tipos que implementam Into
impl<T, U> From<T> for U
where
    T: Into<U>,
{
    fn from(source: T) -> Self {
        source.into()
    }
}

// Convenience trait for bidirectional conversion
pub trait BidirectionalConvertible<T>: Convertible<T> + From<T> + Into<T> {}

// Implementação automática para tipos que implementam From e Into
impl<T, U> BidirectionalConvertible<U> for T where T: Convertible<U> + From<U> + Into<U> {}

// Extension trait for easier conversion
pub trait ConvertExt<T> {
    /// Convert to the target type using the Convertible trait
    fn convert_to(&self) -> T
    where
        Self: Convertible<T>;

    /// Convert from the source type using the Convertible trait
    fn convert_from(source: T) -> Self
    where
        Self: Convertible<T>;
}

impl<T, U> ConvertExt<U> for T
where
    T: Convertible<U>,
{
    fn convert_to(&self) -> U {
        self.to()
    }

    fn convert_from(source: U) -> Self {
        Self::from(source)
    }
}
