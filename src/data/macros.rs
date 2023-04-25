#[macro_export]
macro_rules! extract_value {
    ($row:expr, $field_name:expr, $table_name:expr) => {
        {
            use $crate::datatypes::expect_sql_field;
            $row.get_opt($field_name)
                .unwrap_or_else(|| panic!("{}", expect_sql_field($field_name, $table_name)))
                .unwrap_or_else(|e| panic!("{}: converting {} from {}", e, $field_name, $table_name))
        }
    };
    ($row:expr, $field_name:expr, $table_name:expr, $datatype:ty) => {
        {
            use $crate::datatypes::expect_sql_field;
            $row.get_opt::<$datatype, _>($field_name)
                .unwrap_or_else(|| panic!("{}", expect_sql_field($field_name, $table_name)))
                .unwrap_or_else(|e| panic!("{}: converting {} from {}", e, $field_name, $table_name))
        }
    };
}
#[macro_export]
macro_rules! extract_decimal {
    ($row:expr, $field_name:expr, $table_name:expr) => {
        {
            use $crate::datatypes::expect_sql_field;
            use std::str::FromStr;
            $row.get_opt::<Decimal, _>($field_name)
                .unwrap_or_else(|| panic!("{}", expect_sql_field($field_name, $table_name)))
                .unwrap_or_else(|e| Decimal::from_str(&e.0.as_sql(false)).unwrap_or_default())
        }
    };
}
#[macro_export]
macro_rules! extract_decimal_opt {
    ($row:expr, $field_name:expr, $table_name:expr) => {
        {
            use std::str::FromStr;
            use $crate::datatypes::expect_sql_field;
            $row.get_opt::<Option<Decimal>, _>($field_name)
                .unwrap_or_else(|| panic!("{}", expect_sql_field($field_name, $table_name)))
                .unwrap_or_else(|e| Decimal::from_str(&e.0.as_sql(false)).ok())
        }
    };
}
#[macro_export]
macro_rules! extract_char_opt {
    ($row:expr, $field_name:expr, $table_name:expr) => {
        {
            use $crate::datatypes::expect_sql_field;
            $row.get_opt::<Option<String>, _>($field_name)
                .unwrap_or_else(|| panic!("{}", expect_sql_field($field_name, $table_name)))
                .unwrap_or_else(|e| panic!("{}: converting {} from {}", e, $field_name, $table_name))
                .and_then(|x| x.chars().next())
        }
    };
}
#[macro_export]
macro_rules! extract_bool {
    ($row:expr, $field_name:expr, $table_name:expr) => {
        {
            use $crate::datatypes::expect_sql_field;
            $row.get_opt::<u8, _>($field_name)
                .unwrap_or_else(|| panic!("{}", expect_sql_field($field_name, $table_name)))
                .unwrap_or_else(|e| panic!("{}: converting {} from {}", e, $field_name, $table_name))
                != 0
        }
    };
}
#[macro_export]
macro_rules! extract_bool_opt {
    ($row:expr, $field_name:expr, $table_name:expr) => {
        {
            use $crate::datatypes::expect_sql_field;
            $row.get_opt::<Option<u8>, _>($field_name)
                .unwrap_or_else(|| panic!("{}", expect_sql_field($field_name, $table_name)))
                .unwrap_or_else(|e| panic!("{}: converting {} from {}", e, $field_name, $table_name))
                .map(|x| x != 0)
        }
    };
}

#[macro_export]
macro_rules! new_error {
    ($message:expr, $error_type:expr) => {
        {
            use $crate::datatypes::system_codes::MySystemError;
            logger::MyError::<$crate::datatypes::system_codes::SystemErrorCodes>::new(logger::get_line_info!(), $message, $error_type)
        }
    };
    ($message:expr) => {
        {
            use $crate::datatypes::system_codes::MySystemError;
            logger::MyError::<$crate::datatypes::system_codes::SystemErrorCodes>::new(logger::get_line_info!(), $message, logger::ErrorTypes::Undefined)
        }
    }
}