use rocket::{catch, serde::json::Json, Request};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct DefaultReturnType {
    message: String
}

// Catchers
#[catch(404)]
pub fn not_found(_: &Request) -> Json<DefaultReturnType> {
    return Json(
        DefaultReturnType {
            message: String::from("Not found")
        }
    );
}

#[catch(401)]
pub fn unauthorized(_: &Request) -> Json<DefaultReturnType> {
    return Json(
        DefaultReturnType {
            message: String::from("You're not authorized!")
        }
    );
}

#[catch(429)]
pub fn too_many_requests(_: &Request) -> Json<DefaultReturnType> {
    return Json(
        DefaultReturnType {
            message: String::from("You're limited!")
        }
    );
}
