#[cfg(test)]
mod tests {

    use git2::Repository;

    #[test]
    fn create_new_repo() {
        let path = "test/test_repo";
        let repo = Repository::init(path).expect("Path is valid");
    }
}