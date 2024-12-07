use rocket::{self, get, post, put, delete, serde::json::Json, State, routes};
use rocket::http::Method;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use anyhow::{Result, anyhow};
use core::types::database::Database;
use core::types::schema::{DbValue, DbSchema};
use std::sync::Mutex;
use rocket_cors::{AllowedHeaders, AllowedOrigins, CorsOptions};
use std::time::Duration;
use tokio::time::interval;
use core::io::{save_to_file, load_from_file};
use std::env;
use std::fs;
use dotenv::dotenv;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Record {
    pub id: String,
    pub values: Vec<DbValue>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NewRecord {
    pub values: Vec<DbValue>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateRecord {
    pub values: Vec<DbValue>,
}

pub struct ApiState {
    pub db: Arc<Mutex<Database>>,
    pub db_path: String,
}

pub async fn start_autosave(db: Arc<Mutex<Database>>, db_path: String) {
    let mut interval = interval(Duration::from_secs(30)); // Save every 30 seconds
    
    loop {
        interval.tick().await;
        if let Ok(db) = db.lock() {
            if let Err(e) = save_to_file(&*db, &db_path) {
                eprintln!("Error autosaving database: {}", e);
            }
        }
    }
}

pub fn cors() -> CorsOptions {
    CorsOptions {
        allowed_origins: AllowedOrigins::all(),
        allowed_methods: vec![Method::Get, Method::Post, Method::Put, Method::Delete]
            .into_iter()
            .map(From::from)
            .collect(),
        allow_credentials: true,
        allowed_headers: AllowedHeaders::all(),
        ..Default::default()
    }
}

#[post("/tables/<table_name>", data = "<schema>")]
pub async fn create_table(table_name: &str, schema: Json<DbSchema>, state: &State<ApiState>) -> Result<(), rocket::response::Debug<anyhow::Error>> {
    let mut db = state.db.lock().map_err(|_| anyhow!("Failed to lock database"))?;
    let table = core::types::table::Table::new(table_name.to_string(), schema.into_inner());
    db.add_table(table);
    save_to_file(&*db, &state.db_path)?;
    Ok(())
}

#[get("/tables/<table_name>/records")]
pub async fn get_all(table_name: &str, state: &State<ApiState>) -> Result<Json<Vec<Record>>, rocket::response::Debug<anyhow::Error>> {
    let db = state.db.lock().map_err(|_| anyhow!("Failed to lock database"))?;
    let table = db.get_table(table_name).ok_or_else(|| anyhow!("Table not found"))?;
    
    let records = table.get_rows().into_iter()
        .map(|r| Record {
            id: r.id.to_string(),
            values: r.values.clone(),
        })
        .collect();
    
    Ok(Json(records))
}

#[get("/tables/<table_name>/records/<id>")]
pub async fn get_by_id(table_name: &str, id: &str, state: &State<ApiState>) -> Result<Json<Record>, rocket::response::Debug<anyhow::Error>> {
    let db = state.db.lock().map_err(|_| anyhow!("Failed to lock database"))?;
    let table = db.get_table(table_name).ok_or_else(|| anyhow!("Table not found"))?;
    let id = id.parse::<u32>().map_err(|_| anyhow!("Invalid ID format"))?;
    
    let row = table.get_row(id)?;
    Ok(Json(Record {
        id: row.id.to_string(),
        values: row.values.clone(),
    }))
}

#[post("/tables/<table_name>/records", data = "<record>")]
pub async fn create(table_name: &str, record: Json<NewRecord>, state: &State<ApiState>) -> Result<Json<Record>, rocket::response::Debug<anyhow::Error>> {
    let mut db = state.db.lock().map_err(|_| anyhow!("Failed to lock database"))?;
    let table = db.get_table_mut(table_name).ok_or_else(|| anyhow!("Table not found"))?;
    
    let id = table.insert(record.values.clone())?;
    save_to_file(&*db, &state.db_path)?;
    Ok(Json(Record {
        id: id.to_string(),
        values: record.values.clone(),
    }))
}

#[put("/tables/<table_name>/records/<id>", data = "<record>")]
pub async fn update(table_name: &str, id: &str, record: Json<UpdateRecord>, state: &State<ApiState>) -> Result<Json<Record>, rocket::response::Debug<anyhow::Error>> {
    let mut db = state.db.lock().map_err(|_| anyhow!("Failed to lock database"))?;
    let table = db.get_table_mut(table_name).ok_or_else(|| anyhow!("Table not found"))?;
    let id = id.parse::<u32>().map_err(|_| anyhow!("Invalid ID format"))?;
    
    table.update(id, record.values.clone())?;
    save_to_file(&*db, &state.db_path)?;
    Ok(Json(Record {
        id: id.to_string(),
        values: record.values.clone(),
    }))
}

#[delete("/tables/<table_name>/records/<id>")]
pub async fn delete(table_name: &str, id: &str, state: &State<ApiState>) -> Result<(), rocket::response::Debug<anyhow::Error>> {
    let mut db = state.db.lock().map_err(|_| anyhow!("Failed to lock database"))?;
    let table = db.get_table_mut(table_name).ok_or_else(|| anyhow!("Table not found"))?;
    let id = id.parse::<u32>().map_err(|_| anyhow!("Invalid ID format"))?;
    table.delete(id)?;
    save_to_file(&*db, &state.db_path)?;
    Ok(())
}

#[get("/intersection/<table1>/<table2>")]
pub async fn intersection(table1: &str, table2: &str, state: &State<ApiState>) -> Result<Json<Vec<Record>>, rocket::response::Debug<anyhow::Error>> {
    let db = state.db.lock().map_err(|_| anyhow!("Failed to lock database"))?;
    let table1 = db.get_table(table1).ok_or_else(|| anyhow!("Table 1 not found"))?;
    let table2 = db.get_table(table2).ok_or_else(|| anyhow!("Table 2 not found"))?;
    
    let intersection = table1.intersection(table2)?;
    let records = intersection.into_iter()
        .map(|r| Record {
            id: r.id.to_string(),
            values: r.values.clone(),
        })
        .collect();
    
    Ok(Json(records))
}

#[derive(Debug, Serialize)]
pub struct TableList {
    tables: Vec<String>
}

#[get("/health")]
pub async fn health_check() -> &'static str {
    "OK"
}

#[get("/tables")]
pub async fn list_tables(state: &State<ApiState>) -> Result<Json<TableList>, rocket::response::Debug<anyhow::Error>> {
    let db = state.db.lock().map_err(|_| anyhow!("Failed to lock database"))?;
    let tables = db.tables.iter().map(|t| t.name().to_string()).collect();
    Ok(Json(TableList { tables }))
}

#[get("/tables/<table_name>/details")]
pub async fn get_table_details(table_name: &str, state: &State<ApiState>) -> Result<Json<TableDetails>, rocket::response::Debug<anyhow::Error>> {
    let db = state.db.lock().map_err(|_| anyhow!("Failed to lock database"))?;
    let table = db.get_table(table_name).ok_or_else(|| anyhow!("Table not found"))?;
    
    Ok(Json(TableDetails {
        schema: table.schema.clone(),
        rows: table.get_rows().into_iter()
            .map(|r| Record {
                id: r.id.to_string(),
                values: r.values.clone(),
            })
            .collect(),
    }))
}

#[derive(Debug, Serialize)]
pub struct TableDetails {
    schema: DbSchema,
    rows: Vec<Record>,
}

#[delete("/tables/<table_name>")]
pub async fn delete_table(table_name: &str, state: &State<ApiState>) -> Result<(), rocket::response::Debug<anyhow::Error>> {
    let mut db = state.db.lock().map_err(|_| anyhow!("Failed to lock database"))?;
    db.delete_table(table_name).ok_or_else(|| anyhow!("Table not found"))?;
    save_to_file(&*db, &state.db_path)?;
    Ok(())
}

pub fn rocket() -> rocket::Rocket<rocket::Build> {
    let db_path = env::var("DATABASE_FILE").unwrap_or_else(|_| "database.db".to_string());
    
    // Load existing database or create new one
    let db = if fs::metadata(&db_path).is_ok() {
        load_from_file(&db_path).unwrap_or_else(|_| Database::new(&db_path))
    } else {
        let db = Database::new(&db_path);
        save_to_file(&db, &db_path).unwrap_or_default();
        db
    };
    let db = Arc::new(Mutex::new(db));
    let state = ApiState { db, db_path: db_path.clone() };
    
    let cors = cors().to_cors().expect("Failed to create CORS fairing");
    
    rocket::build()
        .attach(cors)
        .mount("/", routes![health_check])
        .mount("/api", routes![
            list_tables,
            create_table,
            delete_table,
            get_table_details,
            get_all,
            get_by_id,
            create,
            update,
            delete,
            intersection,
        ])
        .manage(state)
}

pub async fn run_server() -> Result<()> {
    dotenv().ok();
    rocket()
        .launch()
        .await
        .map_err(|e| anyhow!("Rocket server error: {}", e))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rocket::local::blocking::Client;
    use rocket::http::{Status, ContentType};
    use core::types::schema::{DbSchema, DbColumn, DbColumnType};

    fn create_test_client() -> Client {
        let db = Arc::new(Mutex::new(Database::new("test.db")));
        let state = ApiState { db, db_path: "test.db".to_string() };
        
        let rocket = rocket::build()
            .mount("/api", routes![
                create_table,
                get_all,
                get_by_id,
                create,
                update,
                delete,
                intersection,
            ])
            .manage(state);
            
        Client::tracked(rocket).expect("valid rocket instance")
    }

    fn create_test_schema() -> DbSchema {
        DbSchema {
            columns: vec![
                DbColumn {
                    name: "id".to_string(),
                    column_type: DbColumnType::Integer,
                },
                DbColumn {
                    name: "name".to_string(),
                    column_type: DbColumnType::String,
                },
                DbColumn {
                    name: "balance".to_string(),
                    column_type: DbColumnType::Money,
                },
            ],
        }
    }

    fn create_test_record() -> Record {
        Record {
            id: "0".to_string(),
            values: vec![
                DbValue::Integer(1),
                DbValue::String("John Doe".to_string()),
                DbValue::Money(1000.0),
            ],
        }
    }

    #[test]
    fn test_create_table() {
        let client = create_test_client();
        let schema = create_test_schema();
        
        let response = client.post("/api/tables/test_table")
            .header(ContentType::JSON)
            .body(serde_json::to_string(&schema).unwrap())
            .dispatch();
            
        assert_eq!(response.status(), Status::Ok);
    }

    #[test]
    fn test_create_and_get_record() {
        let client = create_test_client();
        
        // First create a table
        let schema = create_test_schema();
        client.post("/api/tables/test_table")
            .header(ContentType::JSON)
            .body(serde_json::to_string(&schema).unwrap())
            .dispatch();
            
        // Create a record
        let record = create_test_record();
        let response = client.post("/api/tables/test_table/records")
            .header(ContentType::JSON)
            .body(serde_json::to_string(&record).unwrap())
            .dispatch();
            
        assert_eq!(response.status(), Status::Ok);
        
        // Get the record back
        let response = client.get("/api/tables/test_table/records/0")
            .dispatch();
            
        assert_eq!(response.status(), Status::Ok);
        
        let retrieved_record: Record = serde_json::from_str(
            &response.into_string().unwrap()
        ).unwrap();
        
        assert_eq!(retrieved_record.values, record.values);
    }

    #[test]
    fn test_update_record() {
        let client = create_test_client();
        
        // Create table and initial record
        let schema = create_test_schema();
        client.post("/api/tables/test_table")
            .header(ContentType::JSON)
            .body(serde_json::to_string(&schema).unwrap())
            .dispatch();
            
        let record = create_test_record();
        client.post("/api/tables/test_table/records")
            .header(ContentType::JSON)
            .body(serde_json::to_string(&record).unwrap())
            .dispatch();
            
        // Update the record
        let mut updated_record = record.clone();
        updated_record.values = vec![
            DbValue::Integer(1),
            DbValue::String("Jane Doe".to_string()),
            DbValue::Money(2000.0),
        ];
        
        let response = client.put("/api/tables/test_table/records/0")
            .header(ContentType::JSON)
            .body(serde_json::to_string(&updated_record).unwrap())
            .dispatch();
            
        assert_eq!(response.status(), Status::Ok);
        
        // Verify the update
        let response = client.get("/api/tables/test_table/records/0")
            .dispatch();
            
        assert_eq!(response.status(), Status::Ok);
        
        let retrieved_record: Record = serde_json::from_str(
            &response.into_string().unwrap()
        ).unwrap();
        
        assert_eq!(retrieved_record.values, updated_record.values);
    }

    #[test]
    fn test_delete_record() {
        let client = create_test_client();
        
        // Create table and record
        let schema = create_test_schema();
        client.post("/api/tables/test_table")
            .header(ContentType::JSON)
            .body(serde_json::to_string(&schema).unwrap())
            .dispatch();
            
        let record = create_test_record();
        client.post("/api/tables/test_table/records")
            .header(ContentType::JSON)
            .body(serde_json::to_string(&record).unwrap())
            .dispatch();
            
        // Delete the record
        let response = client.delete("/api/tables/test_table/records/0")
            .dispatch();
            
        assert_eq!(response.status(), Status::Ok);
        
        // Verify deletion
        let response = client.get("/api/tables/test_table/records/0")
            .dispatch();
            
        assert_eq!(response.status(), Status::InternalServerError); // Should fail to find record
    }

    #[test]
    fn test_intersection() {
        let client = create_test_client();
        
        // Create two tables with the same schema
        let schema = create_test_schema();
        client.post("/api/tables/table1")
            .header(ContentType::JSON)
            .body(serde_json::to_string(&schema).unwrap())
            .dispatch();
            
        client.post("/api/tables/table2")
            .header(ContentType::JSON)
            .body(serde_json::to_string(&schema).unwrap())
            .dispatch();
            
        // Add same record to both tables
        let record = create_test_record();
        client.post("/api/tables/table1/records")
            .header(ContentType::JSON)
            .body(serde_json::to_string(&record).unwrap())
            .dispatch();
            
        client.post("/api/tables/table2/records")
            .header(ContentType::JSON)
            .body(serde_json::to_string(&record).unwrap())
            .dispatch();
            
        // Add different record to table2
        let mut different_record = record.clone();
        different_record.values = vec![
            DbValue::Integer(2),
            DbValue::String("Jane Doe".to_string()),
            DbValue::Money(2000.0),
        ];
        
        client.post("/api/tables/table2/records")
            .header(ContentType::JSON)
            .body(serde_json::to_string(&different_record).unwrap())
            .dispatch();
            
        // Get intersection
        let response = client.get("/api/intersection/table1/table2")
            .dispatch();
            
        assert_eq!(response.status(), Status::Ok);
        
        let intersection: Vec<Record> = serde_json::from_str(
            &response.into_string().unwrap()
        ).unwrap();
        
        assert_eq!(intersection.len(), 1);
        assert_eq!(intersection[0].values, record.values);
    }

    #[test]
    fn test_validation_errors() {
        let client = create_test_client();
        
        // Create table
        let schema = create_test_schema();
        client.post("/api/schema/test_table")
            .header(ContentType::JSON)
            .body(serde_json::to_string(&schema).unwrap())
            .dispatch();
            
        // Test wrong number of values
        let mut invalid_record = create_test_record();
        invalid_record.values.pop();
        
        let response = client.post("/api/tables/test_table/records")
            .header(ContentType::JSON)
            .body(serde_json::to_string(&invalid_record).unwrap())
            .dispatch();
            
        assert_eq!(response.status(), Status::InternalServerError);
        
        // Test wrong value type
        let mut invalid_record = create_test_record();
        invalid_record.values[0] = DbValue::String("not an integer".to_string());
        
        let response = client.post("/api/tables/test_table/records")
            .header(ContentType::JSON)
            .body(serde_json::to_string(&invalid_record).unwrap())
            .dispatch();
            
        assert_eq!(response.status(), Status::InternalServerError);
    }
} 