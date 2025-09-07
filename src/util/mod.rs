use rand::{self, Rng};

pub fn generate_token() -> [char; 5] {
    let mut rng = rand::rng();

    let mut token: [char; 5] = [' ', ' ', ' ', ' ', ' '];

    for index in 0..5 {
      let mut choosen_ascii_code = 65u8 + rng.random_range(0..52);
      if choosen_ascii_code >= (65u8 + 26) {
        choosen_ascii_code += 6;
      }
      token[index] = choosen_ascii_code as char;
    }

    return token;
}

use sqlx::{Error, postgres::PgDatabaseError};

pub fn is_duplicated_error(err: &Error) -> bool {
    if let Error::Database(db_err) = err {
        if let Some(pg_err) = db_err.try_downcast_ref::<PgDatabaseError>() {
            return pg_err.code() == "23505";
        }
    }
    false
}
