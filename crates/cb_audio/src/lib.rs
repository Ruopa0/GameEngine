#![allow(clippy::type_complexity, clippy::too_many_arguments, clippy::empty_line_after_doc_comments, clippy::if_same_then_else)]
pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}


