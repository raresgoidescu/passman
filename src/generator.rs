use rand::Rng;

pub fn _generate_password(length: usize, special_chars: bool) -> String {
    const BASE_CHARSET: &str = "QWERTYUIOPASDFGHJKLZXCVBNMqwertyuiopasdfghjklzxcvbnm1234567890";
    const SPECIAL_CHARSET: &str = "!@#$%^&*";

    let charset = if special_chars {
        format!("{}{}", BASE_CHARSET, SPECIAL_CHARSET)
    } else {
        BASE_CHARSET.to_string()
    };

    let mut rng = rand::thread_rng();

    (0..length)
        .map(|_| {
            charset
                .chars()
                .nth(rng.gen_range(0..charset.len()))
                .unwrap()
        })
        .collect()
}
