#![feature(proc_macro_hygiene, decl_macro)]

extern crate reqwest;

use rocket::*;
use rocket_contrib::json::Json;
use stremio_core::types::*;
use stremio_core::types::addons::*;
use stremio_core::types::addons::ResourceResponse::Metas;
use serde::{Deserialize, Serialize};
use serde_json::Value;

const TYPE_STR: &str = "channel";
const API_BASE: &str = "https://www.themealdb.com";
const INVALID_ID: &str = "cooking-invalid-id";
const POSTER_SHAPE: PosterShape = stremio_core::types::PosterShape::Square;

const MANIFEST_RAW: &str = include_str!("../manifest.json");

#[get("/manifest.json")]
fn manifest() -> String {
    MANIFEST_RAW.into()
}

#[get("/catalog/channel/cooking.json")]
fn catalog() -> Option<Json<ResourceResponse>> {
    // @TODO error handling
    let endpoint = String::from("/api/json/v1/1/latest.php");
    Some(Json(
        get_catalog(&endpoint)
            // @TODO fix the unwrap
            .ok()?,
    ))
}

#[get("/catalog/channel/cooking/<genre>")]
fn catalog_genre(genre: String) -> Option<Json<ResourceResponse>> {
    // @TODO from name
    let mut endpoint = String::from("/api/json/v1/1/filter.php?c=");
    let s = genre.to_string().replace("genre=", "").replace(".json", "");

    endpoint.push_str(&s);

    Some(Json(
        get_catalog(&endpoint)
            // @TODO fix the unwrap
            .ok()?,
    ))
}

#[derive(Serialize, Deserialize)]
struct Meals {
	meals: Vec<Value>,
}

fn get_json(data: &str) -> std::result::Result<ResourceResponse, serde_json::error::Error> {
    let json: Meals = serde_json::from_str(&data)?;

    let mut metas = [].to_vec();

    for meal in &json.meals {
    	let meta = MetaPreview {
    		id: format!("cooking:{}", meal["idMeal"].to_string().replace("\"", "")),
	    	type_name: TYPE_STR.to_owned(),
	    	poster: Some(meal["strMealThumb"].to_string().replace("\"", "")),
	    	name: meal["strMeal"].to_string().replace("\"", ""),
	    	poster_shape: POSTER_SHAPE,
    	};

    	metas.push(meta);

    }

    let catalog = Metas {
    	metas: metas
    };

	Ok(catalog)
}

fn get_catalog(endpoint: &str) -> Result<ResourceResponse, reqwest::Error> {

    let url = format!("{}{}", API_BASE, endpoint);

    let json_string = reqwest::get(&url)?
        .text()?;

    let resp = get_json(&json_string)
    			.ok();

	let dummy = Metas {
    	metas: [].to_vec()
    };

    match resp {
        Some(p) => Ok(p),
        None => Ok(dummy),
    }

}

fn main() {
    let cors = rocket_cors::CorsOptions::default().to_cors().unwrap();

    rocket::ignite()
        .mount("/", routes![manifest, catalog, catalog_genre])
        .attach(cors)
        .launch();
}
