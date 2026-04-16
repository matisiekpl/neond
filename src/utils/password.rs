use rand::Rng;

const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
const PASSWORD_LENGTH: usize = 32;

pub fn generate_password() -> String {
    let mut rng = rand::rng();
    (0..PASSWORD_LENGTH)
        .map(|_| {
            let index = rng.random_range(0..CHARSET.len());
            CHARSET[index] as char
        })
        .collect()
}