pub fn camel_to_snake_case(s: &str) -> String {
    if s.len() == 0 {
        return String::new();
    }

    let mut result = String::new();
    let mut is_caps_word = false;
    let mut do_not_underscore = true;

    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() {
            if !is_caps_word {
                // Start of caps word
                if !do_not_underscore {
                    result.push('_');
                }
                result.push(c.to_lowercase().next().unwrap());
                is_caps_word = true;
            } else {
                match s.chars().nth(i + 1) {
                    Some(next_c) if next_c.is_lowercase() => {
                        if !do_not_underscore {
                            result.push('_');
                        }
                        result.push(c.to_lowercase().next().unwrap());
                    }
                    _ => {
                        result.push(c.to_lowercase().next().unwrap());
                    }
                }
            }
        } else {
            result.push(c);
            is_caps_word = false;
        }

        do_not_underscore = c == '_';
    }

    result
}

pub fn camel_to_kebab_case(s: &str) -> String {
    camel_to_snake_case(s).replace("_", "-")
}

fn _snake_to_camel_case(s: &str, start_upper: bool) -> String {
    if s.len() == 0 {
        return String::new();
    }

    let mut result = String::new();
    let mut make_next_char_upper = start_upper;

    for c in s.chars() {
        if c == '_' {
            make_next_char_upper = true;
        } else if make_next_char_upper {
            make_next_char_upper = false;
            result.push(c.to_uppercase().next().unwrap());
        } else {
            make_next_char_upper = false;
            result.push(c);
        }
    }

    result
}

pub fn snake_to_camel_case(s: &str) -> String {
    _snake_to_camel_case(s, false)
}

pub fn snake_to_upper_camel_case(s: &str) -> String {
    _snake_to_camel_case(s, true)
}
