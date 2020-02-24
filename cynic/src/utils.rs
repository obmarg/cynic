pub trait FlattenFrom<T> {
    fn flatten_from(args: T) -> Self;
}

impl<T> FlattenFrom<Option<Vec<Option<T>>>> for Option<Vec<T>> {
    fn flatten_from(args: Option<Vec<Option<T>>>) -> Option<Vec<T>> {
        args.map(|v| v.into_iter().flatten().collect())
    }
}

impl<T> FlattenFrom<Option<Vec<Option<T>>>> for Vec<T> {
    fn flatten_from(args: Option<Vec<Option<T>>>) -> Vec<T> {
        args.map(|v| v.into_iter().flatten().collect())
            .unwrap_or_else(|| vec![])
    }
}

impl<T> FlattenFrom<Option<Vec<T>>> for Vec<T> {
    fn flatten_from(args: Option<Vec<T>>) -> Vec<T> {
        args.unwrap_or_else(|| vec![])
    }
}

impl<T> FlattenFrom<Vec<Option<T>>> for Vec<T> {
    fn flatten_from(args: Vec<Option<T>>) -> Vec<T> {
        args.into_iter().flatten().collect()
    }
}

pub trait FlattenInto<T> {
    fn flatten_into(self) -> T;
}

impl<T, U> FlattenInto<U> for T
where
    U: FlattenFrom<T>,
{
    fn flatten_into(self) -> U {
        U::flatten_from(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flatten_from_opt_vec_opt() {
        let ovo = Some(vec![None, Some(1)]);
        let expected: Vec<i32> = vec![1];
        let output: Vec<i32> = ovo.flatten_into();
        assert_eq!(output, expected);

        let ovo = Some(vec![None, Some(1)]);
        let expected: Option<Vec<i32>> = Some(vec![1]);
        let output: Option<Vec<i32>> = ovo.flatten_into();
        assert_eq!(output, expected);
    }

    #[test]
    fn test_flatten_from_opt_vec() {
        let opt_vec = Some(vec![1]);
        let expected: Vec<i32> = vec![1];
        let output: Vec<i32> = opt_vec.flatten_into();
        assert_eq!(output, expected);

        let opt_vec: Option<Vec<i32>> = None;
        let expected: Vec<i32> = vec![];
        let output: Vec<i32> = opt_vec.flatten_into();
        assert_eq!(output, expected);
    }

    #[test]
    fn test_flatten_from_vec_opt() {
        let opt_vec = vec![Some(1), None];
        let expected: Vec<i32> = vec![1];
        let output: Vec<i32> = opt_vec.flatten_into();
        assert_eq!(output, expected);
    }
}
