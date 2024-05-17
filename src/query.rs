#[macro_export]
macro_rules! delete_wherein {
    ($table:expr, $column:expr, $values:expr) => {{
        let mut query = format!("DELETE FROM {} WHERE {} IN (", $table, $column);
        let mut params: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = Vec::new();

        for (i, value) in $values.iter().enumerate() {
            if i > 0 {
                query.push_str(", ");
            }
            query.push_str(&format!("${}", i + 1));
            params.push(value);
        }

        query.push_str(");");
        (query, params)
    }};
}

#[macro_export]
macro_rules! delete_one {
    ($table:expr, $condition:expr) => {
        format!("DELETE FROM {} WHERE {}", $table, $condition)
    };
}

#[macro_export]
macro_rules! update_query {
    ($table:expr, $set_column:expr, $where_column:expr) => {
        format!(
            "UPDATE {} SET {}=$1 WHERE {}=$2",
            $table, $set_column, $where_column
        )
    };
}
