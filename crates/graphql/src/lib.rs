use std::{sync::Arc, pin::Pin, future::Future, marker::PhantomData};
use async_graphql::{Context, Object, Schema, SimpleObject, ID, InputObject};
use tokio::sync::Mutex;
use core::{types::{database::Database, schema::{DbSchema, DbValue, DbColumnType}, table::Table}, io::{load_from_file, save_to_file}};
use std::env;

pub type GraphQLSchema = Schema<Query, Mutation, async_graphql::EmptySubscription>;

pub struct GraphQLState {
    pub db: Arc<Mutex<Database>>,
    pub db_path: String,
}

#[derive(SimpleObject, InputObject)]
pub struct ColumnInfo {
    name: String,
    column_type: DbColumnType,
}

#[derive(SimpleObject, InputObject)]
pub struct SchemaInfo {
    columns: Vec<ColumnInfo>,
}

impl From<DbSchema> for SchemaInfo {
    fn from(schema: DbSchema) -> Self {
        SchemaInfo {
            columns: schema.columns.into_iter().map(|col| ColumnInfo {
                name: col.name,
                column_type: col.column_type,
            }).collect(),
        }
    }
}

impl From<SchemaInfo> for DbSchema {
    fn from(schema: SchemaInfo) -> Self {
        DbSchema {
            columns: schema.columns.into_iter().map(|col| core::types::schema::DbColumn {
                name: col.name,
                column_type: col.column_type,
            }).collect(),
        }
    }
}

#[derive(SimpleObject)]
pub struct TableInfo {
    name: String,
    schema: SchemaInfo,
}

#[derive(SimpleObject, InputObject)]
pub struct DbValueWrapper {
    integer_value: Option<i32>,
    real_value: Option<f32>,
    char_value: Option<char>,
    string_value: Option<String>,
    money_value: Option<f64>,
    money_range_value: Option<(f64, f64)>,
}

impl From<DbValue> for DbValueWrapper {
    fn from(value: DbValue) -> Self {
        match value {
            DbValue::Integer(i) => DbValueWrapper {
                integer_value: Some(i),
                real_value: None,
                char_value: None,
                string_value: None,
                money_value: None,
                money_range_value: None,
            },
            DbValue::Real(r) => DbValueWrapper {
                integer_value: None,
                real_value: Some(r),
                char_value: None,
                string_value: None,
                money_value: None,
                money_range_value: None,
            },
            DbValue::Char(c) => DbValueWrapper {
                integer_value: None,
                real_value: None,
                char_value: Some(c),
                string_value: None,
                money_value: None,
                money_range_value: None,
            },
            DbValue::String(s) => DbValueWrapper {
                integer_value: None,
                real_value: None,
                char_value: None,
                string_value: Some(s),
                money_value: None,
                money_range_value: None,
            },
            DbValue::Money(m) => DbValueWrapper {
                integer_value: None,
                real_value: None,
                char_value: None,
                string_value: None,
                money_value: Some(m),
                money_range_value: None,
            },
            DbValue::MoneyRange(min, max) => DbValueWrapper {
                integer_value: None,
                real_value: None,
                char_value: None,
                string_value: None,
                money_value: None,
                money_range_value: Some((min, max)),
            },
        }
    }
}

impl From<DbValueWrapper> for DbValue {
    fn from(wrapper: DbValueWrapper) -> Self {
        if let Some(i) = wrapper.integer_value {
            DbValue::Integer(i)
        } else if let Some(r) = wrapper.real_value {
            DbValue::Real(r)
        } else if let Some(c) = wrapper.char_value {
            DbValue::Char(c)
        } else if let Some(s) = wrapper.string_value {
            DbValue::String(s)
        } else if let Some(m) = wrapper.money_value {
            DbValue::Money(m)
        } else if let Some((min, max)) = wrapper.money_range_value {
            DbValue::MoneyRange(min, max)
        } else {
            panic!("Invalid DbValueWrapper: no value set")
        }
    }
}

#[derive(SimpleObject)]
pub struct Record {
    id: ID,
    values: Vec<DbValueWrapper>,
}

pub struct Query;

#[Object]
impl Query {
    async fn tables<'a>(&self, ctx: &'a Context<'_>) -> async_graphql::Result<Vec<String>> {
        let state = ctx.data::<GraphQLState>()?;
        let db = state.db.lock().await;
        Ok(db.tables.iter().map(|t| t.name().to_string()).collect())
    }

    async fn table<'a>(&self, ctx: &'a Context<'_>, name: String) -> async_graphql::Result<Option<TableInfo>> {
        let state = ctx.data::<GraphQLState>()?;
        let db = state.db.lock().await;
        
        if let Some(table) = db.get_table(&name) {
            Ok(Some(TableInfo {
                name: table.name().to_string(),
                schema: table.schema.clone().into(),
            }))
        } else {
            Ok(None)
        }
    }

    async fn records<'a>(&self, ctx: &'a Context<'_>, table_name: String) -> async_graphql::Result<Vec<Record>> {
        let state = ctx.data::<GraphQLState>()?;
        let db = state.db.lock().await;
        
        if let Some(table) = db.get_table(&table_name) {
            Ok(table.get_rows().into_iter().map(|row| Record {
                id: ID(row.id.to_string()),
                values: row.values.into_iter().map(DbValueWrapper::from).collect(),
            }).collect())
        } else {
            Err("Table not found".into())
        }
    }

    async fn record<'a>(&self, ctx: &'a Context<'_>, table_name: String, id: ID) -> async_graphql::Result<Option<Record>> {
        let state = ctx.data::<GraphQLState>()?;
        let db = state.db.lock().await;
        
        if let Some(table) = db.get_table(&table_name) {
            let id: u32 = id.parse()?;
            if let Ok(row) = table.get_row(id) {
                Ok(Some(Record {
                    id: ID(row.id.to_string()),
                    values: row.values.clone().into_iter().map(DbValueWrapper::from).collect(),
                }))
            } else {
                Ok(None)
            }
        } else {
            Err("Table not found".into())
        }
    }
}

pub struct Mutation;

#[Object]
impl Mutation {
    async fn create_table<'a>(&self, ctx: &'a Context<'_>, name: String, schema: SchemaInfo) -> async_graphql::Result<TableInfo> {
        let state = ctx.data::<GraphQLState>()?;
        let mut db = state.db.lock().await;
        
        let table = Table::new(name.clone(), schema.clone().into());
        db.add_table(table);
        
        save_to_file(&*db, &state.db_path)?;
        
        Ok(TableInfo { name, schema })
    }

    async fn insert_record<'a>(&self, ctx: &'a Context<'_>, table_name: String, values: Vec<DbValueWrapper>) -> async_graphql::Result<Record> {
        let state = ctx.data::<GraphQLState>()?;
        let mut db = state.db.lock().await;
        
        if let Some(table) = db.get_table_mut(&table_name) {
            let values: Vec<DbValue> = values.into_iter().map(DbValue::from).collect();
            let id = table.insert(values.clone())?;
            save_to_file(&*db, &state.db_path)?;
            
            Ok(Record {
                id: ID(id.to_string()),
                values: values.into_iter().map(DbValueWrapper::from).collect(),
            })
        } else {
            Err("Table not found".into())
        }
    }

    async fn delete_record<'a>(&self, ctx: &'a Context<'_>, table_name: String, id: ID) -> async_graphql::Result<bool> {
        let state = ctx.data::<GraphQLState>()?;
        let mut db = state.db.lock().await;
        
        if let Some(table) = db.get_table_mut(&table_name) {
            let id: u32 = id.parse()?;
            let result = table.delete(id).is_ok();
            if result {
                save_to_file(&*db, &state.db_path)?;
            }
            Ok(result)
        } else {
            Err("Table not found".into())
        }
    }
}

pub fn create_schema() -> GraphQLSchema {
    let db_path = env::var("DATABASE_PATH").unwrap_or_else(|_| "database.json".to_string());
    let db: Database = load_from_file(&db_path).unwrap_or_else(|_| Database::new("default"));
    
    let state = GraphQLState {
        db: Arc::new(Mutex::new(db)),
        db_path,
    };
    
    Schema::build(Query, Mutation, async_graphql::EmptySubscription)
        .data(state)
        .finish()
} 