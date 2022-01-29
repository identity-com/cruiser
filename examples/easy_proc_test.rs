use solana_generator_derive::test_easy_proc;

#[test_easy_proc]
fn cool() {
    #[cool(boolean_value, count = 3, size = 10, custom_parse cool, many = "hi", many = "wew")]
    fn do_stuff() {}
}
