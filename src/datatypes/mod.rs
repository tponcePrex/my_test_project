pub mod system_codes;
pub mod response_codes;
pub mod system_datatypes;
pub mod structs;

pub fn expect_sql_field(field_name: &'static str, struct_name: &'static str) -> String {
    format!("Expected field {} AT {}", field_name, struct_name)
}