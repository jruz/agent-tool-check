pub struct Output {
    pub denial: Option<Result<serde_json::Value, String>>,
    pub success: bool,
    pub text: String,
}
