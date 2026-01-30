//! Sample repository for testing

#[tokio::main]
async fn main() {
    println!("Hello, HQE!");
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_sample() {
        assert_eq!(2 + 2, 4);
    }
}
