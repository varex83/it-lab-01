use std::hash::Hash;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum DbValue {
    Integer(i32),
    Real(f32),
    Char(char),
    String(String),
    Money(f64),
    MoneyRange(f64, f64),
}

impl Eq for DbValue {}

impl PartialEq for DbValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (DbValue::Integer(a), DbValue::Integer(b)) => a == b,
            (DbValue::Real(a), DbValue::Real(b)) => {
                const EPSILON: f32 = 1e-6;
                (a - b).abs() < EPSILON
            },
            (DbValue::Char(a), DbValue::Char(b)) => a == b,
            (DbValue::String(a), DbValue::String(b)) => a == b,
            (DbValue::Money(a), DbValue::Money(b)) => {
                const EPSILON: f64 = 1e-10;
                (a - b).abs() < EPSILON
            },
            (DbValue::MoneyRange(a1, a2), DbValue::MoneyRange(b1, b2)) => {
                const EPSILON: f64 = 1e-10;
                (a1 - b1).abs() < EPSILON && (a2 - b2).abs() < EPSILON
            },
            _ => false,
        }
    }
}

impl Hash for DbValue {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            DbValue::Integer(i) => i.hash(state),
            DbValue::Real(f) => f.to_bits().hash(state),
            DbValue::Char(c) => c.hash(state),
            DbValue::String(s) => s.hash(state),
            DbValue::Money(m) => m.to_bits().hash(state),
            DbValue::MoneyRange(m1, m2) => {
                m1.to_bits().hash(state);
                m2.to_bits().hash(state);
            }
        }
    }
}

impl DbValue {
    pub fn value_type(&self) -> DbColumnType {
        match self {
            DbValue::Integer(_) => DbColumnType::Integer,
            DbValue::Real(_) => DbColumnType::Real,
            DbValue::Char(_) => DbColumnType::Char,
            DbValue::String(_) => DbColumnType::String,
            DbValue::Money(_) => DbColumnType::Money,
            DbValue::MoneyRange(_, _) => DbColumnType::MoneyRange,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub enum DbColumnType {
    #[serde(rename = "integer")]
    Integer,
    #[serde(rename = "real")]
    Real,
    #[serde(rename = "char")]
    Char,
    #[serde(rename = "string")]
    String,
    #[serde(rename = "money")]
    Money,
    #[serde(rename = "money_range")]
    MoneyRange,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct DbColumn {
    pub name: String,
    pub column_type: DbColumnType,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct DbSchema {
    pub name: String,
    pub columns: Vec<DbColumn>,
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_db_schema_deser() {
        let schema_json = r#"
        {
            "name": "test_schema",
            "columns": [
                {
                    "name": "id",
                    "column_type": "integer"
                },
                {
                    "name": "name",
                    "column_type": "string"
                }
            ]
        }
        "#;


        let schema: DbSchema = serde_json::from_str(schema_json).unwrap();

        assert_eq!(schema, DbSchema {
            name: "test_schema".to_string(),
            columns: vec![
                DbColumn {
                    name: "id".to_string(),
                    column_type: DbColumnType::Integer,
                },
                DbColumn {
                    name: "name".to_string(),
                    column_type: DbColumnType::String,
                }
            ]
        });
    }

    #[test]
    fn test_db_schema_ser() {
        let schema = DbSchema {
            name: "another_yet_schema".to_string(),
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
                    name: "surname".to_string(),
                    column_type: DbColumnType::String,
                }
            ]
        };

        let schema_json = serde_json::to_string(&schema).unwrap();

        assert_eq!(schema_json, r#"{"name":"another_yet_schema","columns":[{"name":"id","column_type":"integer"},{"name":"name","column_type":"string"},{"name":"surname","column_type":"string"}]}"#);
    }

    #[test]
    fn test_db_value_deser() {
        let value_json = r#"
        {
            "Integer": 42
        }
        "#;

        let value: DbValue = serde_json::from_str(value_json).unwrap();

        assert_eq!(value, DbValue::Integer(42));
    }

    #[test]
    fn test_db_value_ser() {
        let value = DbValue::Money(42.0);

        let value_json = serde_json::to_string(&value).unwrap();

        assert_eq!(value_json, r#"{"Money":42.0}"#);
    }
}
