# sansaisummit24
Tool and Jupyter Notebook used in "Enhance Investigations Using LLM, Embeddings, and Clustering" SANS AI Cybersecurity Summit talk.

# Project Structure
`/notebook` - Folder that contains clustering Jupyter notebook and associated module and requirements.

`/cluster-commands-rs` - Folder that contains the Rust source for the demo'd cluster-commands.exe tool.

# Building the Rust tool
It is asumed you already have Rust installed on your machine. 
cd into the `/cluster-commands-rs` folder and run `cargo build --release`.

The binary exe will be under `/cluster-commands-rs/target/release`. Give the
command a `-h` for help prompt.