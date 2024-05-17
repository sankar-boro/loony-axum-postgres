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
macro_rules! delete_where {
    ($table:expr, $column:expr, $value:expr) => {
        format!("DELETE FROM {} WHERE {}={}", $table, $column, $value)
    };
}
