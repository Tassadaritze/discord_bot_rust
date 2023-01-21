use std::collections::HashMap;
use std::fs;

use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Markov {
    data: HashMap<String, Vec<String>>,
    start_keys: Vec<String>,
    key_size: u8,
}

impl Markov {
    pub fn new(key_size: u8, path: &str, should_import: bool) -> Self {
        let start_time = std::time::Instant::now();

        if should_import {
            if let Ok(val) = fs::read("markov_data") {
                match rmp_serde::from_slice::<Markov>(&val) {
                    Ok(val) => {
                        println!(
                            "Markov data serialized in: {:?}",
                            std::time::Instant::now().duration_since(start_time)
                        );
                        if key_size != val.key_size {
                            println!("Asked to construct a Markov instance with key_size {}, but imported one with {} instead.", key_size, val.key_size);
                        }
                        dbg!(val.start_keys.len());
                        return val;
                    }
                    Err(err) => eprintln!("couldn't serialize found markov data: {err}"),
                }
            }
        }

        let mut markov: HashMap<String, Vec<String>> = HashMap::new();
        let mut start_keys: Vec<String> = vec![];
        for line in fs::read_to_string(path)
            .expect("couldn't read message dump")
            .trim_end()
            .split('\n')
        {
            let words: Vec<&str> = line.split_whitespace().collect();
            for i in 0..words.len().saturating_sub(key_size as usize) {
                let mut key = words[i].to_string();
                for word in words.iter().take(i + key_size as usize).skip(i + 1) {
                    key += " ";
                    key += word;
                }
                if i == 0 {
                    start_keys.push(key.clone());
                }
                let value = words[i + key_size as usize];
                if let Some(val) = markov.get_mut(&key) {
                    val.push(value.to_string());
                } else {
                    markov.insert(key, vec![value.to_string()]);
                }
            }
        }

        println!(
            "Markov data built in: {:?}",
            std::time::Instant::now().duration_since(start_time)
        );

        let markov = Self {
            data: markov,
            start_keys,
            key_size,
        };

        match rmp_serde::to_vec(&markov) {
            Ok(val) => match fs::write("markov_data", val) {
                Ok(_) => (),
                Err(err) => eprintln!("couldn't write markov data: {err}"),
            },
            Err(err) => eprintln!("couldn't serialize markov data: {err}"),
        };

        markov
    }

    pub async fn generate_string(&self) -> String {
        const MAX_LEN: u16 = 1600;

        let mut key = self
            .start_keys
            .get(thread_rng().gen_range(0..self.start_keys.len()))
            .unwrap()
            .split(' ')
            .collect::<Vec<&str>>();

        let mut out: Vec<&str> = key.clone();

        while let Some(val) = self.data.get(&*key.join(" ")) {
            if out.len() > MAX_LEN as usize {
                break;
            }
            out.push(&val[thread_rng().gen_range(0..val.len())]);
            key = Vec::from(&out[out.len().saturating_sub(self.key_size as usize)..out.len()]);
        }

        out.join(" ")
    }
}
