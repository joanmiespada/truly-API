use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};

pub fn generate_random_path(len: usize) -> String {
    thread_rng()
        .sample_iter(&Alphanumeric)
        .take(len)
        .map(char::from)
        .collect::<String>()
}

pub fn generate_random_url() -> String {
    let random_path = generate_random_path(10);
    let random_id = generate_random_path(5);
    format!("https://example.com/{}/{}", random_path, random_id)
}

pub fn generate_random_email() -> String {
    use rnglib::{RNG, Language};
    let rng = RNG::try_from(&Language::Elven).unwrap();
    let first_name = rng.generate_name();
    let last_name = rng.generate_name();
    format!("{}.{}@example.com",first_name,last_name )
}