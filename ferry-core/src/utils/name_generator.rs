use names::Generator;

/// Wrapper to generate a random name for anything (e.g, client, server etc.)
pub fn get_random_name() -> String {
    let mut generator = Generator::default();
    generator.next().expect("Failed to generate random name")
}