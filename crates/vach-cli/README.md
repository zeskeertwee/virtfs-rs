# **`vach-cli`**

`vach-cli` is a simple CLI for packing, unpacking and handling `.vach` files.

> For the [`vach20`](https://crates.io/crates/vach/0.2.3) version use [this version](https://crates.io/crates/vach-cli/0.3.3) of the CLI instead, this version of the CLI only works with [`vach30`](https://crates.io/crates/vach/0.3.5) onwards

---

## **Installation**

```sh
cargo install vach-cli
```

## **Usage:**

Generally follows the template:

```sh
vach [subcommand] -[key] [...values]
```

```sh
# List all entries in the archive "source.vach"
vach list -i source.vach

# Pack the files hello.png, click.wav and dialogue.txt into assets.vach
vach pack -i hello.png click.wav dialogue.tx -o assets.vach

# Pack all the file in the directory textures into textures.vach and apply compression
vach pack -d textures -o textures.vach -c

# This lists out the contents of textures.vach
vach list -i textures.vach
┌───────────────────────────┬───────┬────────────┐
│          id               │ size  │   flags    │
├───────────────────────────┼───────┼────────────┤
│   textures/perlin.png     │ 698 B │ Flags[C--] │
│    textures/dirt.png      │ 391 B │ Flags[C--] │
│   textures/cobble.png     │ 733 B │ Flags[C--] │
│    textures/map.json      │ 311 B │ Flags[C--] │
└───────────────────────────┴───────┴────────────┘
```

---

## **Commands:**

- Running `vach help` will list all the commands. They each have their own inputs.

- Also check out `spec/main.txt` for the official spec.

- You can also run `vach [command] --help` to see help on each individual command.

- Because of how the CLI works, the positions of the keys does not matter, so:

   `vach list -i target.vach -m CMYKV` and `vach list -m CMYKV -i target.vach` are equivalent.

---

### 1: pack

> `pack` is used to pack files and directories into archives. It takes inputs for customizing how the archive should be packaged.

```sh
# Any pack command must have an output, set using the "-o" or "--output" keys
# This builds an empty archive
vach pack -o hello.vach

# You can add files as inputs using the "-i" or "--input" keys
vach pack -o hello.vach -i hello.txt goodbye.txt

# Or add a directory using "-d" or "--directory"
vach pack -o hello.vach -d ./hello

# Add a directory recursively using "-r" or "--directory-r"
vach pack -o hello.vach -r ./hello

# Inputs can be added in tandem
vach pack -o hello.vach -i hi.txt bye.txt -d greetings PR -r talks

# Exclude a given file from the queue
vach pack -x hello/secret.txt -o hello.vach -d hello

# Provide a keypair or secret key for cryptographic use
vach pack -k keypair.kp -o hello.vach -i hello.txt goodbye.txt
vach pack -s secret_key.sk -o hello.vach -i hello.txt goodbye.txt

### MODIFIERS ####
# Compression: "-c always", "-c never" or "-c detect"
vach pack -c always -o hello.vach -i hello.txt goodbye.txt
vach pack -c never -o hello.vach -i hello.txt goodbye.txt

# CompressionAlgorithm: "-g lz4", "-g snappy" or "-g brotli". Both "-g" and "--compress-algo" keys work
vach pack -g lz4 -c always -o hello.vach -i hello.txt goodbye.txt

# Note compression has been set to never here so setting the compression algorithm to be used has no effect
vach pack -g snappy -c never -o hello.vach -i hello.txt goodbye.txt

# Hash: "-a" or "--hash"
# Whether to include signatures in the archive
# This help to detect if the archive has been tampered with
# But it's very computationally intensive so use them sparingly
vach pack -s -o hello.vach -i hello.txt goodbye.txt

# Encrypt: "-e" or "--encrypt"
# Whether to encrypt your archive
# If no pre-existing keypair|secret_key is provided then a new one is written: `${OUTPUT_ARCHIVE}.kp`
# EG hello.vach -> hello.vach.kp, same applies for "-a"
vach pack -e -o hello.vach -i hello.txt goodbye.txt

# Flags: "-f" or "--flags"
# Flags set into the Archive header
# Here the flags are set to 0b1000_1100_1001_0000
vach pack -f 35984 -o hello.vach -i hello.txt goodbye.txt

# Magic: "-m" or "--magic"
# Make your archive unique by setting a custom MAGIC
vach pack -m CMYKX -o hello.vach -i hello.txt goodbye.txt

# Truncate: "-t" or "--truncate"
# This modifier deletes the original files once they are packaged
# hello.txt & goodbye.txt are now deleted
vach pack -t -o hello.vach -i hello.txt goodbye.txt
```

### 2: unpack

>`unpack` it's just like `pack` but backwards

```sh
# Provide an input: "-i" or "--input"
vach unpack -i target.vach

# Output directory: "-o" or "--output"
# Specify where to unpack the archive
vach unpack -i source.vach -o ./output/

# Specify what magic your archive uses
# Magic: "-m" or "--magic"
vach unpack -m `CMYKX -i source.vach

# Truncate: "-t" or "--truncate"
# Deletes the original archive after unpacking
vach unpack -t -i source.vach

# If the archive is encrypted then provide a keypair or public key
vach unpack -k keypair.kp -i source.vach
vach unpack -s keypair.sk -i source.vach
```

### 3: pipe

>`pipe`: Read the data from a _specific_ entry and pipe it to stdout

```sh
# Print to stdout
vach pipe -i target.vach -r npc-dialogue.txt

# Pipe directly into a file
vach pipe -i target.vach -r npc-dialogue.txt >> npc-dialogue.txt

# Pipe into another process' stdin
vach pipe -i presets.vach -r low.json | jq '."TextureResolution"'
```

### 4: list

> Lists all the entries in the archive as a table

```sh
# Provide some input: "-i" or "--input"
vach list -i textures.vach

# MAGIC: "-m" or "--magic"
# If the archive uses a custom magic
vach list -i textures.vach -m TXTRS

# SORT: "--sort"
# How to sort the entries inside the table
# Can either be: size-ascending, size-descending, alphabetical, alphabetical-reversed
vach list -i textures.vach -m TXTRS --sort size-descending
```

### 5: verify

> Verifies the validity of a file as an archive

```sh
# Simplest command
vach verify -i textures.vach

# MAGIC: "-m" or "--magic"
vach verify -i textures.vach -m TXTRS
```

### 6: keypair

> Key-pair generation command

```sh
vach keypair -o keypair.kp
# -> keypair.kp

# Splits the keypair into it's secret and public components immediately after generation
vach keypair -s -o keypair.kp

# -> keypair.pk
# -> keypair.sk
```

### 7: split

> Splits an existing keypair into it's public and secret components

```sh
vach split -i keypair.kp

# -> keypair.pk
# -> keypair.sk
```
