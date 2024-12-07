use serde::{Deserialize, Serialize};
use crate::types::table::Table;

#[derive(Debug, Serialize, Deserialize)]
pub struct Database {
    pub name: String,
    pub tables: Vec<Table>,
}

impl Database {
    pub fn new(name: &str) -> Self {
        Database {
            name: name.to_string(),
            tables: Vec::new(),
        }
    }

    pub fn add_table(&mut self, table: Table) {
        self.tables.push(table);
    }

    pub fn get_table(&self, name: &str) -> Option<&Table> {
        self.tables.iter().find(|t| t.name() == name)
    }

    pub fn get_table_mut(&mut self, name: &str) -> Option<&mut Table> {
        self.tables.iter_mut().find(|t| t.name() == name)
    }

    pub fn delete_table(&mut self, name: &str) -> Option<Table> {
        let index = self.tables.iter().position(|t| t.name() == name);
        index.map(|i| self.tables.remove(i))
    }
}

#[cfg(test)]
mod tests {
    use crate::types::table::create_test_table;
    use super::*;

    #[test]
    fn test_database() {
        let mut db = Database::new("test_db");

        let table1 = create_test_table("table1");
        let table2 = create_test_table("table2");

        db.add_table(table1.clone());
        db.add_table(table2.clone());

        assert_eq!(db.get_table("table1"), Some(&table1));
        assert_eq!(db.get_table("table2"), Some(&table2));

        let table3 = create_test_table("table3");
        db.add_table(table3.clone());

        assert_eq!(db.get_table("table3"), Some(&table3));

        let table4 = create_test_table("table4");
        db.add_table(table4.clone());

        assert_eq!(db.get_table("table4"), Some(&table4));

        assert_eq!(db.delete_table("table4"), Some(table4));
        assert_eq!(db.get_table("table4"), None);
    }
}
