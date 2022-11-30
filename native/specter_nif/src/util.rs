use uuid::Uuid;

pub fn gen_uuid() -> String {
    Uuid::new_v4().hyphenated().to_string()
}
