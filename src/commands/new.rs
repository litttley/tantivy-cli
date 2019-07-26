use ansi_term::Colour::{Blue, Green, Red};
use ansi_term::Style;
use clap::ArgMatches;
use serde_json;
use std::convert::From;
use std::io;
use std::io::Write;
use std::path::PathBuf;
use tantivy;
use tantivy::schema::Cardinality;
use tantivy::schema::*;
use tantivy::Index;
use std::fs;
use cang_jie::{CangJieTokenizer, TokenizerOption, CANG_JIE};
use jieba_rs::Jieba;
use std::{collections::HashSet,  iter::FromIterator, sync::Arc};

pub fn run_new_cli(matches: &ArgMatches) -> Result<(), String> {
    let index_directory = PathBuf::from(matches.value_of("index").unwrap());
    run_new(index_directory).map_err(|e| format!("{:?}", e))
}

fn prompt_input<P: Fn(&str) -> Result<(), String>>(prompt_text: &str, predicate: P) -> String {
    loop {
        print!(
            "{prompt_text:<width$} ? ",
            prompt_text = Style::new().bold().fg(Blue).paint(prompt_text),
            width = 40
        );
        io::stdout().flush().unwrap();
        let mut buffer = String::new();
        io::stdin()
            .read_line(&mut buffer)
            .ok()
            .expect("Failed to read line");
        let answer = buffer.trim_end().to_string();
        match predicate(&answer) {
            Ok(()) => {
                return answer;
            }
            Err(msg) => {
                println!("Error: {}", Style::new().bold().fg(Red).paint(msg));
            }
        }
    }
}

fn field_name_validate(field_name: &str) -> Result<(), String> {
    if is_valid_field_name(field_name) {
        Ok(())
    } else {
        Err(String::from(
            "Field name must match the pattern [_a-zA-Z0-9]+",
        ))
    }
}

fn prompt_options(msg: &str, codes: Vec<char>) -> char {
    let options_string: Vec<String> = codes.iter().map(|c| format!("{}", c)).collect();
    let options = options_string.join("/");
    let predicate = |entry: &str| {
        if entry.len() != 1 {
            return Err(format!("Invalid input. Options are ({})", options));
        }
        let c = entry.chars().next().unwrap().to_ascii_uppercase();
        if codes.contains(&c) {
            return Ok(());
        } else {
            return Err(format!("Invalid input. Options are ({})", options));
        }
    };
    let message = format!("{} ({})", msg, options);
    let entry = prompt_input(&message, predicate);
    entry.chars().next().unwrap().to_ascii_uppercase()
}

fn prompt_yn(msg: &str) -> bool {
    prompt_options(msg, vec!['Y', 'N']) == 'Y'
}

fn ask_add_field_text(field_name: &str, schema_builder: &mut SchemaBuilder) {
    let text_field_indexing = TextFieldIndexing::default()
        .set_tokenizer(CANG_JIE)
        .set_index_option(IndexRecordOption::WithFreqsAndPositions);
    let mut text_options = TextOptions::default().set_indexing_options(text_field_indexing);
    if prompt_yn("Should the field be stored") {
        text_options = text_options.set_stored();
    }

    if prompt_yn("Should the field be indexed") {
        let mut text_indexing_options = TextFieldIndexing::default()
            .set_index_option(IndexRecordOption::Basic)
            .set_tokenizer(CANG_JIE);

        if prompt_yn("Should the term be tokenized?") {
            if prompt_yn("Should the term frequencies (per doc) be in the index") {
                if prompt_yn("Should the term positions (per doc) be in the index") {
                    text_indexing_options = text_indexing_options
                        .set_index_option(IndexRecordOption::WithFreqsAndPositions);
                } else {
                    text_indexing_options =
                        text_indexing_options.set_index_option(IndexRecordOption::WithFreqs);
                }
            }
        } else {
            text_indexing_options = text_indexing_options.set_tokenizer(CANG_JIE);
        }

        text_options = text_options.set_indexing_options(text_indexing_options);
    }

    schema_builder.add_text_field(field_name, text_options);
}

fn ask_add_field_u64(field_name: &str, schema_builder: &mut SchemaBuilder) {
    let mut u64_options = IntOptions::default();
    if prompt_yn("Should the field be stored") {
        u64_options = u64_options.set_stored();
    }
    if prompt_yn("Should the field be fast") {
        u64_options = u64_options.set_fast(Cardinality::SingleValue);
    }
    if prompt_yn("Should the field be indexed") {
        u64_options = u64_options.set_indexed();
    }
    schema_builder.add_u64_field(field_name, u64_options);
}

fn ask_add_field(schema_builder: &mut SchemaBuilder) {
    println!("\n\n");
    let field_name = prompt_input("New field name ", field_name_validate);
    let text_or_integer = prompt_options("Text or unsigned 32-bit integer", vec!['T', 'I']);
    if text_or_integer == 'T' {
        ask_add_field_text(&field_name, schema_builder);
    } else {
        ask_add_field_u64(&field_name, schema_builder);
    }
}

fn run_new(directory: PathBuf) -> tantivy::Result<()> {
    println!(
        "\n{} ",
        Style::new().bold().fg(Green).paint("Creating new index")
    );
    println!(
        "{} ",
        Style::new()
            .bold()
            .fg(Green)
            .paint("Let's define it's schema!")
    );
    let mut schema_builder = SchemaBuilder::default();
    loop {
        ask_add_field(&mut schema_builder);
        if !prompt_yn("Add another field") {
            break;
        }
    }
    let schema = schema_builder.build();
    let schema_json = format!("{}", serde_json::to_string_pretty(&schema).unwrap());
    println!("\n{}\n", Style::new().fg(Green).paint(schema_json));
    match fs::create_dir(&directory) {
        Ok(_) => (),
        // Proceed here; actual existence of index is checked in Index::create_in_dir
        Err(ref e) if e.kind() == io::ErrorKind::AlreadyExists => (),
        Err(e) => panic!("{:?}", e),
    };
    Index::create_in_dir(&directory, schema)?;
    Ok(())
}
