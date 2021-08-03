# variants:
#   1:144852532_144852533insCCC
#   1:144852537T>C
#   1:144852545C>T
#   1:144852606G>A
#   1:144852633_144852634del

cargo run -- \
	-b 'tests/test.1:144852532-144852632.bam' \
	-1 '1:144852532_144852533insCCC' -2 '1:144852537T>C'

cargo run -- \
	-b 'tests/test.1:144852532-144852632.bam' \
	-1 '1:144852532_144852533insCCC' -2 '1:144852545C>T'

cargo run -- \
	-b 'tests/test.1:144852532-144852632.bam' \
	-1 '1:144852532_144852533insCCC' -2 '1:144852606G>A'

cargo run -- \
	-b 'tests/test.1:144852532-144852632.bam' \
	-1 '1:144852532_144852533insCCC' -2 '1:144852633_144852634del'

cargo run -- \
	-b 'tests/test.1:144852532-144852632.bam' \
	-1 '1:144852537T>C' -2 '1:144852545C>T'

cargo run -- \
	-b 'tests/test.1:144852532-144852632.bam' \
	-1 '1:144852537T>C' -2 '1:144852606G>A'

cargo run -- \
	-b 'tests/test.1:144852532-144852632.bam' \
	-1 '1:144852537T>C' -2 '1:144852633_144852634del'

cargo run -- \
	-b 'tests/test.1:144852532-144852632.bam' \
	-1 '1:144852545C>T' -2 '1:144852606G>A'

cargo run -- \
	-b 'tests/test.1:144852532-144852632.bam' \
	-1 '1:144852545C>T' -2 '1:144852633_144852634del'

cargo run -- \
	-b 'tests/test.1:144852532-144852632.bam' \
	-1 '1:144852606G>A' -2 '1:144852633_144852634del'

cargo run -- \
	-b 'tests/test.1:144852532-144852632.bam' \
	-1 '1:144852633_144852634del' -2 '1:144852606G>A'
