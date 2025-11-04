// Simple keypair converter
use bs58;
use std::fs::File;
use std::io::Write;

fn main() {
    let private_key_b58 = "YOUR_BASE58_PRIVATE_KEY_HERE";

    // Decode base58 to bytes
    let decoded = bs58::decode(private_key_b58)
        .into_vec()
        .expect("Failed to decode base58");

    // Convert to JSON array format
    let json = format!("{:?}", decoded);

    // Write to keypair.json
    let mut file = File::create("keypair.json").expect("Failed to create file");
    file.write_all(json.as_bytes()).expect("Failed to write file");

    println!("✅ keypair.json created!");
}
