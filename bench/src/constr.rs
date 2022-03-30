use std::convert::TryFrom;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::path::Path;
use std::time::Instant;

use clap::Parser;

#[derive(Parser, Debug)]
#[clap(name = "constr", about = "A program to measure.")]
struct Args {
    #[clap(short = 'k', long)]
    keys_filename: String,
}

fn main() {
    let args = Args::parse();

    println!("keys_filename\t{}", &args.keys_filename);
    let mut keys = load_file(&args.keys_filename);
    keys.sort_unstable();
    show_memory_stats(&keys);
}

fn show_memory_stats(keys: &[String]) {
    println!("#keys: {}", keys.len());
    {
        println!("[crawdad/trie]");
        let start = Instant::now();
        let trie = crawdad::builder::Builder::new()
            .from_keys(keys)
            .release_trie();
        let duration = start.elapsed();
        print_memory("heap_bytes", trie.heap_bytes());
        println!("num_elems: {}", trie.num_elems());
        println!("vacant_ratio: {:.3}", trie.vacant_ratio());
        println!("constr_sec: {:.3}", duration.as_secs_f64());
    }
    {
        println!("[crawdad/mptrie]");
        let start = Instant::now();
        let trie = crawdad::builder::Builder::new()
            .minimal_prefix()
            .from_keys(keys)
            .release_mptrie();
        let duration = start.elapsed();
        print_memory("heap_bytes", trie.heap_bytes());
        println!("num_elems: {}", trie.num_elems());
        println!("vacant_ratio: {:.3}", trie.vacant_ratio());
        println!("constr_sec: {:.3}", duration.as_secs_f64());
    }
    {
        println!("[crawdad/mpftrie]");
        let start = Instant::now();
        let trie = crawdad::builder::Builder::new()
            .minimal_prefix()
            .from_keys(keys)
            .release_mpftrie();
        let duration = start.elapsed();
        print_memory("heap_bytes", trie.heap_bytes());
        println!("num_elems: {}", trie.num_elems());
        println!("vacant_ratio: {:.3}", trie.vacant_ratio());
        println!("constr_sec: {:.3}", duration.as_secs_f64());
    }
    {
        println!("[yada]");
        let start = Instant::now();
        let data = yada::builder::DoubleArrayBuilder::build(
            &keys
                .iter()
                .cloned()
                .enumerate()
                .map(|(i, key)| (key, u32::try_from(i).unwrap()))
                .collect::<Vec<_>>(),
        )
        .unwrap();
        let duration = start.elapsed();
        print_memory("heap_bytes", data.len());
        println!("constr_sec: {:.3}", duration.as_secs_f64());
    }
}

fn print_memory(title: &str, bytes: usize) {
    println!(
        "{}: {} bytes, {:.3} MiB",
        title,
        bytes,
        bytes as f64 / (1024.0 * 1024.0)
    );
}

fn load_file<P>(path: P) -> Vec<String>
where
    P: AsRef<Path>,
{
    let file = File::open(path).unwrap();
    let buf = BufReader::new(file);
    buf.lines().map(|line| line.unwrap()).collect()
}
