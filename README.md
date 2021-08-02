# Varlink

Get cis or trans linkage of two variants.

# Getting Started

Cli help message.

```shell
$ varlink
varlink 0.1.0
Xiaochuan Liu<sean.lyo@outlook.com>
Cis/trans linkage calculator for variant pair.

USAGE:
    varlink [FLAGS] --first <first> --second <second> --bam <bam>

FLAGS:
    -h, --help       Prints help information
        --merge      When *merge* variant of the target exists, do not count read as a support.
    -V, --version    Prints version information

OPTIONS:
    -b, --bam <bam>          Bam file path.
    -1, --first <first>      First variant, in HGVS format.
    -2, --second <second>    Second variant, in HGVS format.
```

Example.

```shell
$ varlink -b 'tests/test.1:144852532-144852632.bam' -1 '1:144852545C>T' -2 '1:144852537T>C'
{
  "both": 0,
  "first": 829,
  "second": 425,
  "neither": 849
}
```
