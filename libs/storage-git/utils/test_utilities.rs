#[macro_export]
macro_rules! assert_value_eq_json {
    ($value:expr, $($json:tt)*) => {
        let v: ::serde_json::Value = ::serde_json::json!($($json)*);

        let left = ::serde_json::to_string(&$value).unwrap();
        let right = ::serde_json::to_string(&v).unwrap();

        assert_eq!(left, right, "json are not equals");
    };
}

#[macro_export]
macro_rules! tasks_vec {
    ($($json:tt)*) => {{
        let val = ::serde_json::json!($($json)*);
        let data: Vec<Task> = ::serde_json::from_value(val).unwrap();
        data
    }};
}
