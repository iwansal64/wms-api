use rocket::{catch, Request};

// Catchers
#[catch(404)]
pub fn not_found(_: &Request) -> String {
    return String::from("Not found");
}

#[catch(401)]
pub fn unauthorized(_: &Request) -> String {
    return String::from("You're not authorized!");
}

#[catch(429)]
pub fn too_many_requests(_: &Request) -> String {
    return String::from("You're limited!");
}
