pub fn load_themes() -> syntect::highlighting::ThemeSet {
    syntect::highlighting::ThemeSet::load_defaults()
}

pub fn load_syntaxes() -> syntect::parsing::SyntaxSet {
    syntect::parsing::SyntaxSet::load_defaults_newlines()
}
