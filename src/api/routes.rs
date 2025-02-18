use crate::state::State;
use actix_web::{HttpResponse, Responder, delete, post, web};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize)]
pub struct CreateLinkRequest {
    url: String,
}

#[derive(Serialize)]
pub struct CreateLinkResponse {
    id: String,
    url: String,
}

#[derive(Serialize)]
pub struct ErrorResponse {
    error: String,
}

#[post("/links")]
async fn create_link(state: web::Data<State>, req: web::Json<CreateLinkRequest>) -> impl Responder {
    // Generate a UUID and take first 8 chars for a shorter ID
    // may need to stick with full uuid for future release to avoid clashes
    let full_uuid = Uuid::new_v4();
    let id = full_uuid.to_string()[..8].to_string();

    let client = match state.database_client().await {
        Ok(client) => client,
        Err(_) => {
            return HttpResponse::InternalServerError()
                .json(ErrorResponse { error: "Database connection failed".to_string() });
        }
    };

    match crate::database::create_link(&client, &id, &req.url).await {
        Ok(_) => HttpResponse::Created().json(CreateLinkResponse { id, url: req.url.clone() }),
        Err(_) => HttpResponse::InternalServerError()
            .json(ErrorResponse { error: "Failed to create link".to_string() }),
    }
}

#[delete("/links/{id}")]
async fn delete_link(state: web::Data<State>, path: web::Path<String>) -> impl Responder {
    let id = path.into_inner();

    // Validate that the ID is the correct length (8 characters)
    if id.len() != 8 {
        return HttpResponse::BadRequest()
            .json(ErrorResponse { error: "Invalid ID format".to_string() });
    }

    let client = match state.database_client().await {
        Ok(client) => client,
        Err(_) => {
            return HttpResponse::InternalServerError()
                .json(ErrorResponse { error: "Database connection failed".to_string() });
        }
    };

    match crate::database::delete_link(&client, &id).await {
        Ok(_) => HttpResponse::NoContent().finish(),
        Err(_) => HttpResponse::InternalServerError()
            .json(ErrorResponse { error: "Failed to delete link".to_string() }),
    }
}
