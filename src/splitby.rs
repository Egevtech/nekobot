pub trait SplitBy {
    fn split_by(self, max: usize) -> Vec<Self> where Self: Sized;
}

impl SplitBy for String {
    fn split_by(self, max: usize) -> Vec<Self> {
        if self.len() < max {
            return vec![self]
        }

        let chars = self.chars().collect::<Vec<char>>();

        let mut i: usize = 0usize;
        let mut blocksize: usize = 0usize;
        let mut res: Vec<String> = Vec::with_capacity(2);
        
        while i < chars.len() {
            let mut block = String::new();

            while blocksize < max && i < chars.len() {
                block.push(chars[i]);
                blocksize += 1;
                i += 1;
            }

            blocksize = 0;

            res.push(block);
        }
    
        res
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! str {
        ($str:expr) => {
            String::from($str)
        }
    }

    #[test]
    fn splitby_test1() {
        let str = str!("12345678");
        let strs = str.split_by(4);

        assert_eq!(strs, [str!("1234"), str!("5678"), str!("90")]);
    }
}