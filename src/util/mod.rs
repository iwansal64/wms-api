use rand::{self, Rng};

pub fn generate_token() -> [char; 5] {
  let mut rng = rand::rng();
  
  let mut token: [char; 5] = [' ', ' ', ' ', ' ', ' '];

  for index in 0..5 {
    token[index] = (65u8 + rng.random_range(0..52)) as char;
  }

  return token;
}
