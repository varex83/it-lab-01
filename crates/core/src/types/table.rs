use std::collections::{HashMap, HashSet};
use anyhow::bail;
use serde::{Deserialize, Serialize};
use crate::types::schema::{DbSchema, DbValue};
#[cfg(test)]
use crate::types::schema::{DbColumn, DbColumnType};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Row {
    pub id: u32,
    pub values: Vec<DbValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Table {
    pub schema: DbSchema,
    pub rows: HashMap<u32, Row>,
    pub index: u32,
    pub name: String,
}

impl Table {
    pub fn new(name: String, schema: DbSchema) -> Self {
        Table {
            schema,
            rows: HashMap::new(),
            index: 0,
            name,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn insert(&mut self, row: Vec<DbValue>) -> anyhow::Result<u32> {
        self.validate(&row)?;

        let id = self.index;

        self.rows.insert(id, Row {
            id,
            values: row,
        });

        self.index += 1;
        Ok(self.index - 1)
    }

    pub fn delete(&mut self, id: u32) -> anyhow::Result<()> {
        self.rows.remove(&id).ok_or_else(|| anyhow::anyhow!("Row not found"))?;
        Ok(())
    }

    pub fn update(&mut self, id: u32, new_row: Vec<DbValue>) -> anyhow::Result<()> {
        self.validate(&new_row)?;

        let row = self.get_row_mut(id);
        row.values = new_row;
        Ok(())
    }

    pub fn intersection(&self, other: &Table) -> anyhow::Result<Vec<Row>> {
        if self.schema != other.schema {
            bail!("Schemas do not match");
        }

        let mut result = Vec::new();

        let mut unique_rows = HashSet::new();

        for row in self.rows.values() {
            unique_rows.insert(row.clone().values);
        }

        for row in other.rows.values() {
            if unique_rows.contains(&row.values) {
                result.push(row.clone());
            }
        }

        Ok(result)
    }

    pub fn validate(&self, row: &[DbValue]) -> anyhow::Result<()> {
        if row.len() != self.schema.columns.len() {
            bail!("Row length does not match schema length");
        }

        for (i, value) in row.iter().enumerate() {
            if value.value_type() != self.schema.columns[i].column_type {
                bail!("Value type does not match schema type");
            }
        }

        Ok(())
    }

    pub fn get_row(&self, id: u32) -> anyhow::Result<&Row> {
        self.rows.get(&id).ok_or_else(|| anyhow::anyhow!("Row not found"))
    }

    pub fn get_row_mut(&mut self, id: u32) -> &mut Row {
        self.rows.get_mut(&id).ok_or_else(|| anyhow::anyhow!("Row not found")).unwrap()
    }

    pub fn get_rows(&self) -> Vec<Row> {
        self.rows.values().cloned().collect()
    }
}

#[cfg(test)]
pub fn create_test_schema() -> DbSchema {
    DbSchema {
        columns: vec![
            DbColumn {
                name: "col1".to_string(),
                column_type: DbColumnType::Integer,
            },
            DbColumn {
                name: "col2".to_string(),
                column_type: DbColumnType::String,
            },
        ],
    }
}

#[cfg(test)]
pub fn create_test_row() -> Vec<DbValue> {
    vec![
        DbValue::Integer(42),
        DbValue::String("test".to_string()),
    ]
}

#[cfg(test)]
pub fn create_test_table(name: &str) -> Table {
    Table::new(
        name.to_string(),
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
        ],
    })
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn test_table_creation() {
        let schema = create_test_schema();
        let table = Table::new("test_table".to_string(), schema.clone());
        assert_eq!(table.name(), "test_table");
        assert_eq!(table.schema, schema);
        assert!(table.rows.is_empty());
        assert_eq!(table.index, 0);
    }

    #[test]
    fn test_insert_valid_row() {
        let mut table = Table::new("test_table".to_string(), create_test_schema());
        let row = create_test_row();
        let id = table.insert(row.clone()).unwrap();

        assert_eq!(id, 0);
        assert_eq!(table.rows.len(), 1);
        assert_eq!(table.get_row(0).unwrap().values, row);
    }

    #[test]
    fn test_insert_invalid_row_length() {
        let mut table = Table::new("test_table".to_string(), create_test_schema());
        let result = table.insert(vec![DbValue::Integer(42)]);
        assert!(result.is_err());
    }

    #[test]
    fn test_insert_invalid_type() {
        let mut table = Table::new("test_table".to_string(), create_test_schema());
        let result = table.insert(vec![
            DbValue::String("wrong".to_string()),
            DbValue::String("test".to_string()),
        ]);
        assert!(result.is_err());
    }

    #[test]
    fn test_delete_existing_row() {
        let mut table = Table::new("test_table".to_string(), create_test_schema());
        let id = table.insert(create_test_row()).unwrap();
        assert!(table.delete(id).is_ok());
        assert!(table.rows.is_empty());
    }

    #[test]
    fn test_delete_nonexistent_row() {
        let mut table = Table::new("test_table".to_string(), create_test_schema());
        assert!(table.delete(0).is_err());
    }

    #[test]
    fn test_update_existing_row() {
        let mut table = Table::new("test_table".to_string(), create_test_schema());
        let id = table.insert(create_test_row()).unwrap();

        let new_row = vec![
            DbValue::Integer(99),
            DbValue::String("updated".to_string()),
        ];

        assert!(table.update(id, new_row.clone()).is_ok());
        assert_eq!(table.get_row(id).unwrap().values, new_row);
    }

    #[test]
    fn test_intersection() {
        let mut table1 = Table::new("test_table".to_string(), create_test_schema());
        let mut table2 = Table::new("test_table".to_string(), create_test_schema());

        let row1 = create_test_row();
        let row2 = vec![
            DbValue::Integer(99),
            DbValue::String("different".to_string()),
        ];

        table1.insert(row1.clone()).unwrap();
        table1.insert(row2.clone()).unwrap();
        table2.insert(row1.clone()).unwrap();

        let intersection = table1.intersection(&table2).unwrap();

        assert_eq!(intersection.len(), 1);
        assert_eq!(intersection[0].values, row1);
    }

    #[test]
    fn test_intersection_different_schemas() {
        let schema1 = DbSchema {
            columns: vec![
                DbColumn {
                    name: "id".to_string(),
                    column_type: DbColumnType::Integer,
                },
                DbColumn {
                    name: "name".to_string(),
                    column_type: DbColumnType::String,
                },
            ],
        };

        let schema2 = DbSchema {
            columns: vec![
                DbColumn {
                    name: "id".to_string(),
                    column_type: DbColumnType::Integer,
                },
                DbColumn {
                    name: "age".to_string(),
                    column_type: DbColumnType::Integer,
                },
            ],
        };

        let table1 = Table::new("test_table".to_string(), schema1);
        let table2 = Table::new("test_table".to_string(), schema2);

        assert!(table1.intersection(&table2).is_err());
    }

    #[test]
    fn test_get_rows() {
        let mut table = Table::new("test_table".to_string(), create_test_schema());
        let row1 = create_test_row();
        let row2 = vec![
            DbValue::Integer(99),
            DbValue::String("different".to_string()),
        ];

        table.insert(row1.clone()).unwrap();
        table.insert(row2.clone()).unwrap();

        let rows = table.get_rows();
        assert_eq!(rows.len(), 2);
        assert!(rows.iter().any(|r| r.values == row1));
        assert!(rows.iter().any(|r| r.values == row2));
    }
}
