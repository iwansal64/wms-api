use rand::{self, Rng};

pub fn generate_token(length: u8) -> String {
    let mut rng = rand::rng();

    let mut token: String = String::new();

    for _ in 0..length {
      let mut choosen_ascii_code = 65u8 + rng.random_range(0..52);
      if choosen_ascii_code >= (65u8 + 26) {
        choosen_ascii_code += 6;
      }
      token += (choosen_ascii_code as char).to_string().as_str();
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
