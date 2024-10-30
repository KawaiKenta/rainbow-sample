use serde::{Deserialize, Serialize};
use sha1::{Digest, Sha1};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{self, BufRead, Write};

// レインボーチェーンの長さ
const CHAIN_LENGTH: usize = 300;
const RAINBOW_TABLE_FILE: &str = "rainbow_table.json";

// レインボーテーブルの型定義（シリアライズ用）
#[derive(Serialize, Deserialize)]
struct RainbowTable {
    // 16進数文字列のハッシュ値をキーとし、プレインテキストを値とする構造に変更
    table: HashMap<String, String>,
}

// SHA-1ハッシュを計算
fn hash(input: &str) -> Vec<u8> {
    let mut hasher = Sha1::new();
    hasher.update(input.as_bytes());
    hasher.finalize().to_vec()
}

fn reduce(hash: &[u8], position: usize) -> String {
    let mut num = u32::from_be_bytes([hash[0], hash[1], hash[2], hash[3]]) ^ (position as u32);
    num = num.wrapping_add(u32::from_be_bytes([hash[4], hash[5], hash[6], hash[7]]));

    let charset = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    let charset_len = charset.len() as u32;

    let length = 6 + (num % 3) as usize;

    let mut result = String::new();
    for _ in 0..length {
        let idx = (num % charset_len) as usize;
        result.push(charset[idx] as char);
        num /= charset_len;
    }

    result
}

// ファイルからパスワードリストを読み込み、レインボーテーブルを生成
fn generate_rainbow_table() -> io::Result<RainbowTable> {
    let mut table = HashMap::new();
    let file = File::open("list.txt")?;
    let reader = io::BufReader::new(file);

    for (i, line) in reader.lines().enumerate() {
        let start_text = line?;
        let mut plaintext = start_text.clone();
        for j in 0..CHAIN_LENGTH {
            plaintext = reduce(&hash(&plaintext), j);
        }
        let end_hash = hash(&plaintext);
        let end_hash_hex = hex::encode(end_hash); // ハッシュを16進数文字列に変換
        table.insert(end_hash_hex, start_text);

        // 進捗表示
        if i % 1000 == 0 {
            println!("\rレインボーテーブルを生成中... {}行目", i);
        }
    }

    Ok(RainbowTable { table })
}

// レインボーテーブルをJSON形式で保存
fn save_rainbow_table(rainbow_table: &RainbowTable) -> io::Result<()> {
    let file = File::create(RAINBOW_TABLE_FILE)?;
    serde_json::to_writer(file, rainbow_table)?;
    Ok(())
}

// JSON形式のレインボーテーブルをファイルからロード
fn load_rainbow_table() -> io::Result<RainbowTable> {
    let file = File::open(RAINBOW_TABLE_FILE)?;
    let rainbow_table = serde_json::from_reader(file)?;
    Ok(rainbow_table)
}

// ハッシュ値からプレインテキストを復元
fn crack_hash(rainbow_table: &RainbowTable, target_hash: &str) -> Option<String> {
    let target_bytes = hex::decode(target_hash).expect("無効なハッシュ形式です");

    // チェーンの逆方向から探索
    for i in (0..CHAIN_LENGTH).rev() {
        let mut current_hash = target_bytes.clone();

        // 各ステップでリダクションとハッシュを繰り返し、テーブル内のエントリと照合
        for j in i..CHAIN_LENGTH {
            let candidate_text = reduce(&current_hash, j);
            let hashed_candidate = hash(&candidate_text);

            // レインボーテーブルで一致するエントリがあるか確認
            if let Some(start_text) = rainbow_table.table.get(&hex::encode(&hashed_candidate)) {
                let mut plaintext = start_text.clone();

                // 一致した場合、チェーンを開始からたどり、ターゲットハッシュと一致するか確認
                for k in 0..CHAIN_LENGTH {
                    if hash(&plaintext) == target_bytes {
                        return Some(plaintext);
                    }
                    plaintext = reduce(&hash(&plaintext), k);
                }
            }

            // 一致が見つからない場合、次のリダクションを生成してハッシュを更新
            current_hash = hash(&reduce(&current_hash, j));
        }
    }

    None // 一致するプレインテキストが見つからない場合
}

fn main() -> io::Result<()> {
    let rainbow_table = if fs::metadata(RAINBOW_TABLE_FILE).is_ok() {
        println!("既存のレインボーテーブルをロードしています...");
        load_rainbow_table()?
    } else {
        println!("新しいレインボーテーブルを生成しています...");
        let table = generate_rainbow_table()?;
        save_rainbow_table(&table)?;
        table
    };
    println!("レインボーテーブルのロードが完了しました");

    // 適当な文字列 "casper4"　をハッシュ化・reduceしてみる -> Vuvk5CAA
    // これのハッシュ値 0da49c9a507b3a983d1804a675ae8cb9422746d7

    let target_hash = "0da49c9a507b3a983d1804a675ae8cb9422746d7";
    if let Some(plaintext) = crack_hash(&rainbow_table, target_hash) {
        println!("ハッシュ値からプレインテキストを特定: {}", plaintext);
    } else {
        println!("一致するプレインテキストが見つかりませんでした");
    }

    Ok(())
}
