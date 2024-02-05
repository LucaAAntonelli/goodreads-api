
pub mod lib {
    pub fn add(left: usize, right: usize) -> usize {
        left + right
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = lib::add(2, 2);
        assert_eq!(result, 4);
    }
}
