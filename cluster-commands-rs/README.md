# Build
`cargo build --release`

# cluster-commands
```
> .\cluster-commands.exe -h
A tool that can extract commands from EVTX files and summarize clusters. Currently this tool only extracts commands that are found in the Event.EventData.CommandLine attribute

Usage: cluster-commands.exe [OPTIONS] --source <SOURCE> --csv-output <CSV_OUTPUT> --cache <CACHE>

Options:
  -s, --source <SOURCE>
          The source that contains EVTX records
  -c, --csv-output <CSV_OUTPUT>
          The csv output file to write output to
  -c, --cache <CACHE>
          The embeddings cache directory
      --openai-token <OPENAI_TOKEN>
          OpenAI API token. If not used, the OPENAI_KEY env var will be used or an error will be thrown
      --embedding-model <EMBEDDING_MODEL>
          Embedding model selection [default: text-embedding-3-small] [possible values: text-embedding-3-small, text-embedding-3-large, text-embedding-ada-002]
      --embedding-dimensions <EMBEDDING_DIMENSIONS>
          Embedding model selection
      --cluster-tolerance <CLUSTER_TOLERANCE>
          Set the clustering tolerance threshold [default: 0.5]
      --cluster-grouping <CLUSTER_GROUPING>
          Set the cluster grouping threshold [default: 2]
      --logging <LOGGING>
          The logging level to use [default: Info] [possible values: Off, Error, Warn, Info, Debug, Trace]
  -h, --help
          Print help (see more with '--help')
  -V, --version
          Print version
```