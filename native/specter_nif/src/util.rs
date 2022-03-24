use uuid::Uuid;

pub fn gen_uuid() -> String {
    Uuid::new_v4().to_hyphenated().to_string()
}
