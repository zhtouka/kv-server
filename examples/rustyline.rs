use kvserver::CommandRequest;
use rustyline::error::ReadlineError;
use rustyline::{DefaultEditor, Result};

fn main() -> Result<()> {
    // `()` can be used when no completer is required
    let mut rl = DefaultEditor::new()?;
    #[cfg(feature = "with-file-history")]
    if rl.load_history("history.txt").is_err() {
        println!("No previous history.");
    }
    loop {
        let readline = rl.readline("kvserver>> ");
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str())?;
                let s = line.split_whitespace();
                let cmd = get_cmd(s);
                println!("cmd: {:?}", cmd);
            },
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break
            },
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break
            },
            Err(err) => {
                println!("Error: {:?}", err);
                break
            }
        }
    }
    #[cfg(feature = "with-file-history")]
    rl.save_history("history.txt");
    Ok(())
}


fn get_cmd<'a>(mut iter: impl Iterator<Item = &'a str>) -> CommandRequest {
    let op = iter.next().unwrap();
    match op.to_ascii_uppercase().as_ref() {
        "GET" => {
            let (table, key) = (iter.next().unwrap(), iter.next().unwrap());
            CommandRequest::new_hget(table, key)
        },
        "SET" => {
            let (table, key) = (iter.next().unwrap(), iter.next().unwrap());
            let value = iter.next().unwrap();
            CommandRequest::new_hset(table, key, value.into())
        },
        "DELETE" => {
            let (table, key) = (iter.next().unwrap(), iter.next().unwrap());
            CommandRequest::new_hdelete(table, key)
        },
        _ => CommandRequest::new_hexists("t1", "k1"),
    }
}