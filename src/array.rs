//! Utility functions related to arrays.

pub fn array_from_iterator<I: Copy + Default, T: IntoIterator<Item = I>, const N: usize>(
    iterator: T,
) -> [I; N] {
    let mut output = [I::default(); N];
    iterator
        .into_iter()
        .take(N)
        .enumerate()
        .for_each(|(index, value)| {
            output[index] = value;
        });
    output
}
