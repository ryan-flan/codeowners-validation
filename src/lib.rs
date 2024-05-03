pub mod parser;

pub mod validators {
    pub(crate) mod avoid_shadowing;
    pub(crate) mod duplicate_patterns;
    pub(crate) mod exists;
    pub mod validator;
}
