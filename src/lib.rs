pub fn greeting<'a>() -> &'a str {
    "Welcome to Firefly engine!"
}

#[cfg(test)]
mod tests {
    use crate::greeting;

    #[test]
    fn greeting_message() {
        assert_eq!(greeting(), "Welcome to Firefly engine!");
    }
}
