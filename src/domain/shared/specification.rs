trait Specification<T> {
    fn is_satisfied_by(&self, item: &T) -> bool;
}
