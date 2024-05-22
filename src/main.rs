use regex::Regex;

fn main() {
    let file_name = "file.cpp";
    let mdb_name = "mdb.mdb";

    let mdb = std::fs::read_to_string(mdb_name).unwrap();
    let buffer = std::fs::read_to_string(file_name).unwrap();
    let mut new_buffer = buffer.clone();
    for line in buffer.lines() {
        let string = parse_line(&line, &mdb);
        match string {
            None => {}
            Some(s) => {
                new_buffer = new_buffer.replacen(line, &s, 1);
            }
        }
    }

    std::fs::write(&format!("{}_new", file_name), &new_buffer).unwrap()
}

fn parse_line(line: &str, mdb: &str) -> Option<String> {
    let re =
        Regex::new(r#"\"\s*(?P<Err>Err[0-9])+\s*\"\s*(?P<Strings>(?:<<\s*\S+\s*)*);"#).unwrap();
    let cap = re.captures(line);
    let cap = match cap {
        None => return None,
        Some(_) => cap.unwrap(),
    };
    let err = &cap["Err"];
    let strings = &cap["Strings"];

    let re = Regex::new(r#"(\w+)"#).unwrap();
    let mut strings_vec = vec![];
    for (_, [string]) in re.captures_iter(strings).map(|c| c.extract()) {
        strings_vec.push(string);
    }

    let mut mdb_match = String::new();
    for mdb_line in mdb.lines() {
        if mdb_line.starts_with(err) {
            mdb_match = get_mdb(mdb_line);
        }
    }

    if (mdb_match).is_empty() {
        return None;
    }

    let mut new_line = format!("QString mdb_message = QString({})", mdb_match);
    for string in strings_vec {
        new_line = format!("{}.arg({})", new_line, string);
    }
    new_line.push(';');

    if line.contains("qCritical") {
        new_line.push_str(" qCritical() << mdb_message;");
    } else if line.contains("qInfo") {
        new_line.push_str(" qInfo() << mdb_message;");
    } else if line.contains("qWarning") {
        new_line.push_str(" qWarning() << mdb_message;");
    };
    let re = Regex::new(r#"(?P<spaces>.*)(?:qCritical|qInfo|qWarning)"#).unwrap();
    let caps = re.captures(line).unwrap();
    let spaces = &caps["spaces"];
    new_line = format!("{}{}", spaces, new_line);

    println!(
        "***Replace \n{}\n***with \n{}\n\n (yes/no):",
        line, new_line
    );
    let mut answer = String::new();
    std::io::stdin().read_line(&mut answer).unwrap();
    answer = answer.trim().to_string();
    if "no".contains(&answer) {
        return None;
    }

    Some(new_line)
}

fn get_mdb(mdb: &str) -> String {
    let re = Regex::new(r#"\w+\s+(?P<mdb>\".+\")"#).unwrap();
    let cap = re.captures(mdb).unwrap();
    let mdb_result = cap["mdb"].to_string();

    let mut prev_is_percent = false;
    let mut iterator = 0;
    let mdb_result: String = mdb_result
        .chars()
        .map(|x| match x {
            '%' => {
                prev_is_percent = true;
                '%'
            }
            's' => {
                if prev_is_percent {
                    prev_is_percent = false;
                    iterator += 1;
                    iterator.to_string().chars().next().unwrap()
                } else {
                    's'
                }
            }
            _ => x,
        })
        .collect();
    mdb_result
}
