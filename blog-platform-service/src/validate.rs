use rocket::data::{self, Data, FromData};
use rocket::http::Status;
use rocket::serde::{Deserialize, json::Json};
use validator::Validate;

pub struct Validated<T>(pub T);

#[rocket::async_trait]
impl<'r, T> FromData<'r> for Validated<T>
where
    T: Deserialize<'r> + Validate,
{
    type Error = String;

    async fn from_data(req: &'r rocket::Request<'_>, data: Data<'r>) -> data::Outcome<'r, Self> {
        // 1. Read the data into a Json object (limit to 1MB)
        let outcome = Json::<T>::from_data(req, data).await;

        match outcome {
            data::Outcome::Success(json) => {
                // 2. Run the validation logic
                match json.validate() {
                    Ok(_) => data::Outcome::Success(Validated(json.into_inner())),
                    Err(e) => data::Outcome::Error((Status::UnprocessableEntity, format!("{}", e))),
                }
            }
            data::Outcome::Error((status, _)) => {
                data::Outcome::Error((status, "Invalid JSON format".into()))
            }
            data::Outcome::Forward(f) => data::Outcome::Forward(f),
        }
    }
}
