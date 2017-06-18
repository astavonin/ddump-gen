#[macro_use]
extern crate clap;
extern crate clang;
#[macro_use]
extern crate log;
extern crate env_logger;

use clang::*;
use std::path::Path;

fn gen_dump_op(stream_type: &str, obj_type: &str, name: &str, fields: &[String]) {
    println!("friend {}& operator<<({}& os, const {}& obj);",
             stream_type,
             stream_type,
             obj_type);
    println!("{}& operator<<({}& os, const {}& obj) {{",
             stream_type,
             stream_type,
             obj_type);
    println!("\tos << \"{} (\"", name);
    let fcount = fields.len();
    for (i, f) in fields.iter().enumerate() {
        print!("\t\t<< \"{}: \" << {}", f, f);
        if i == fcount - 1 {
            println!("");
        } else {
            println!(" << \", \"");
        }
    }
    println!("\t<< \")\";");
    println!("\treturn os;\n}}")
}

fn gen_fields_list(ent: Entity, name: &str) -> Option<(String, Vec<String>)> {

    match ent.get_location() {
        None => {}
        Some(s) => {
            if !s.is_in_main_file() {
                return None;
            }
        }
    }

    let mut res = None;
    if ent.get_display_name().unwrap_or("".to_string()) == name &&
       (ent.get_kind() == EntityKind::ClassDecl || ent.get_kind() == EntityKind::StructDecl) {

        let mut fields = Vec::new();

        for sub_ent in ent.get_children().into_iter() {
            if sub_ent.get_kind() == EntityKind::FieldDecl {
                fields.push(sub_ent.get_display_name().unwrap_or("unknown".to_string()));
            }
        }
        res = Some((ent.get_type().unwrap().get_display_name(), fields));
    } else {
        for sub_ent in ent.get_children().into_iter() {
            res = gen_fields_list(sub_ent, name);
        }
    }
    return res;
}

fn process(path: &str, name: &str, std: &str, stream_type: &str) {
    let clang = Clang::new().unwrap();
    let index = Index::new(&clang, false, false);
    let tu = match index
              .parser(path)
              .arguments(&[format!("-std=c++{}", std)])
              .parse() {
        Err(err) => {
            error!("Unable to parse {}: {}", path, err);
            return;
        }
        Ok(v) => v,
    };

    match gen_fields_list(tu.get_entity(), name) {
        None => error!("class/struct {} did not find in {}", name, path),
        Some((obj_type, fields)) => gen_dump_op(stream_type, &obj_type, name, &fields),
    }
}

fn main() {
    env_logger::init().unwrap();

    let matches = clap_app!(myapp =>
                            (about: "Debug dump generator for C++ classes/structures.")
                            (version: env!("CARGO_PKG_VERSION"))
                            (author: env!("CARGO_PKG_AUTHORS"))
                            (@arg SRC_FILE: -s --source_file +takes_value +required
                             "The shource (header) file with declaration")
                            (@arg CLASS_NAME: -c --class +takes_value +required
                             "Class/structure name for generation")
                            (@arg STD_VER: --std +takes_value "C++ standard (11 by default)")
                            (@arg OSTREAM_TYPE: -o --ostream +takes_value
                             "Output stream type. Supported types QDebug, std::ostream. (std::ostream by default)")
                           ).get_matches();

    let path = matches.value_of("SRC_FILE").unwrap();
    let name = matches.value_of("CLASS_NAME").unwrap();
    let std = matches.value_of("STD_VER").unwrap_or("11");
    let ostream = matches.value_of("OSTREAM_TYPE").unwrap_or("std::ostream");
    if !Path::new(path).exists() {
        error!("Unable to find {}", path);
        return;
    }
    process(path, name, std, ostream)
}
