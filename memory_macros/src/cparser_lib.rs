use std::ops::Range;

use regex::Regex;

#[derive(Debug, Clone)]
pub struct CField {
    pub ctype: String,
    pub name: String,
    pub offset: String,
    pub bit_size: Option<usize>,
    pub offset_range: Range<usize>,
}

#[derive(Debug, Clone)]
pub struct CStruct {
    pub name: String,
    pub inherit: Option<String>,
    pub fields: Vec<CField>,
}

// this specifies 2 different ways this can pick upp offsets, first is the basic struct definition and the second is how dumps.host does constexpr
// I know that you should not really do these things with regex, however it works
lazy_static::lazy_static! {
    static ref FIELD_REGEX: Regex = Regex::new(r#"([\w<>* ,]+?) (\w+)( : (\d)|); // (0[xX][0-9a-fA-F]*)"#).unwrap();
    static ref STRUCT_NO_INHERIT_REGEX: Regex = Regex::new(r#"struct ([A-Za-z0-9_]*)\s*\{([\w\W]*?)\};"#).unwrap();
    static ref STRUCT_INHERIT_REGEX: Regex = Regex::new(r#"struct ([A-Za-z0-9_]*) : ([A-Za-z0-9_]*)\s*\{([\w\W]*?)\};"#).unwrap();
    static ref STRUCT_CONSTEXPR_INHERIT_REGEX: Regex =Regex::new(r#"Inheritance: (\w*)[\w\W]*?namespace ([A-Za-z0-9_]*)\W*\{([\w\W]*?)\}"#).unwrap();
    static ref STRUCT_CONSTEXPR_NO_INHERIT_REGEX: Regex = Regex::new(r#"Inheritance: NONE[\w\W]*?namespace ([A-Za-z0-9_]*)\W*\{([\w\W]*?)\}"#).unwrap();
    static ref FIELD_CONSTEXPR_REGEX: Regex = Regex::new(r"constexpr auto (\w*) = (0x[0-9a-f]*); // (.*)").unwrap();
    static ref PTR_REGEX: Regex = Regex::new(r#"(\w*)\*"#).unwrap();
}

fn get_fields(content: &str, field_start: usize) -> Vec<CField> {
    FIELD_REGEX
        .captures_iter(content)
        .map(|captures| CField {
            ctype: captures[1].to_owned(),
            name: captures[2].to_owned(),
            offset: captures[5].to_owned(),
            bit_size: captures
                .get(4)
                .map(|x| x.as_str().parse().expect("Bad regex")),
            offset_range: {
                let cap = captures.get(5).unwrap();
                cap.start() + field_start..cap.end() + field_start
            },
        })
        .collect()
}

fn get_fields_constexpr(content: &str, field_start: usize) -> Vec<CField> {
    FIELD_CONSTEXPR_REGEX
        .captures_iter(content)
        .map(|captures| CField {
            ctype: captures[3].to_owned(),
            name: captures[1].to_owned(),
            offset: captures[2].to_owned(),
            offset_range: {
                let cap = captures.get(2).unwrap();
                cap.start() + field_start..cap.end() + field_start
            },
            bit_size: None,
        })
        .collect()
}

pub fn get_rust_name(name: &str) -> Option<String> {
    if name.contains(":") || name.contains("[") {
        return None;
    }

    let mut mod_str = name.replace("struct ", "").trim().to_owned();

    // be aware, order matter
    let matches = vec![
        ("char", "i8"),
        ("uint8_t", "u8"),
        ("uint16_t", "u16"),
        ("uint32_t", "u32"),
        ("uint64_t", "u64"),
        ("int8_t", "i8"),
        ("int16_t", "i16"),
        ("int32_t", "i32"),
        ("int64_t", "i64"),
        ("uint8", "u8"),
        ("uint16", "u16"),
        ("uint32", "u32"),
        ("uint64", "u64"),
        ("int8", "i8"),
        ("int16", "i16"),
        ("int32", "i32"),
        ("int64", "i64"),
        ("uint", "u32"),
        ("int", "i32"),
        ("float", "f32"),
        ("double", "f64"),
    ];

    for (a, b) in matches {
        mod_str = mod_str.replace(a, b);
    }

    mod_str = PTR_REGEX.replace_all(&mod_str, "${1}Ptr").to_string();

    Some(mod_str)
}

pub fn to_snake_case(input: &str) -> String {
    let match_first_cap = Regex::new("(.)([A-Z][a-z]+)").unwrap();
    let match_all_cap = Regex::new("([a-z0-9])([A-Z])").unwrap();

    let l1 = match_first_cap.replace_all(input, "${1}_${2}");
    let l2 = match_all_cap.replace_all(&l1, "${1}_${2}");
    l2.to_lowercase()
}

/** returns a list of all structs + byte offset in string */
pub fn parse_cfile(file: &str) -> Vec<(usize, CStruct)> {
    let mut structs: Vec<(usize, CStruct)> = STRUCT_INHERIT_REGEX
        .captures_iter(file)
        .map(|captures| {
            (
                captures.get(0).unwrap().start(),
                CStruct {
                    name: captures[1].to_owned(),
                    inherit: Some(captures[2].to_owned()),
                    fields: get_fields(&captures[3], captures.get(3).unwrap().start()),
                },
            )
        })
        .collect();
    structs.append(
        &mut STRUCT_NO_INHERIT_REGEX
            .captures_iter(file)
            .map(|captures| {
                (
                    captures.get(0).unwrap().start(),
                    CStruct {
                        name: captures[1].to_owned(),
                        inherit: None,
                        fields: get_fields(&captures[2], captures.get(2).unwrap().start()),
                    },
                )
            })
            .collect(),
    );

    structs.append(
        &mut STRUCT_CONSTEXPR_INHERIT_REGEX
            .captures_iter(file)
            .filter(|c| c[1].trim() != "NONE")
            .map(|captures| {
                (
                    captures.get(0).unwrap().start(),
                    CStruct {
                        name: captures[2].to_owned(),
                        inherit: Some(captures[1].to_owned()),
                        fields: get_fields_constexpr(
                            &captures[3],
                            captures.get(3).unwrap().start(),
                        ),
                    },
                )
            })
            .collect(),
    );
    structs.append(
        &mut STRUCT_CONSTEXPR_NO_INHERIT_REGEX
            .captures_iter(file)
            .map(|captures| {
                (
                    captures.get(0).unwrap().start(),
                    CStruct {
                        name: captures[1].to_owned(),
                        inherit: None,
                        fields: get_fields_constexpr(
                            &captures[2],
                            captures.get(2).unwrap().start(),
                        ),
                    },
                )
            })
            .collect(),
    );

    structs.sort_by(|(a, _), (b, _)| a.cmp(b));
    structs
}
