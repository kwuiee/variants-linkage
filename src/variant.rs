use std::io;
use std::str::FromStr;

mod noms {
    pub(super) use nom::bytes::complete::{is_a, tag, take_until};
    pub(super) use nom::character::complete::digit1;
    pub(super) use nom::combinator::opt;
    pub(super) use nom::error::{Error, ErrorKind};
    pub(super) use nom::sequence::tuple;
    pub(super) use nom::{Err, IResult};
}

#[derive(Debug, PartialEq)]
pub enum Edit {
    // Substitution
    Sub,
    // Deletion
    Del,
    // Insertion
    Ins,
    // Deletion-insertion
    Delins,
    // Identity
    Identity,
}

#[derive(Debug, PartialEq)]
pub enum Format {
    Hgvs,
    Vcf,
}

impl FromStr for Format {
    type Err = io::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "hgvs" => Ok(Self::Hgvs),
            "vcf" => Ok(Self::Vcf),
            _ => Err(io::Error::new(io::ErrorKind::InvalidData, s)),
        }
    }
}

impl Edit {
    pub fn is_del(&self) -> bool {
        matches!(self, &Self::Del)
    }

    pub fn is_ins(&self) -> bool {
        matches!(self, &Self::Ins)
    }

    pub fn is_sub(&self) -> bool {
        matches!(self, &Self::Sub)
    }

    pub fn is_delins(&self) -> bool {
        matches!(self, &Self::Delins)
    }

    pub fn is_identity(&self) -> bool {
        matches!(self, &Self::Identity)
    }
}

#[derive(Debug, PartialEq)]
pub struct Variant {
    contig: String,
    start: u32,
    end: u32,
    edit: Edit,
    refseq: Option<String>,
    altseq: Option<String>,
}

fn parse_contig(input: &str) -> noms::IResult<&str, String> {
    let (res, contig) = noms::take_until(":")(input)?;
    Ok((res, contig.to_string()))
}

fn parse_position(input: &str) -> noms::IResult<&str, (u32, u32)> {
    let (res, start) = noms::digit1(input)?;
    let start = start.parse::<u32>().unwrap();
    let (res, end) = noms::opt(noms::tuple((noms::tag("_"), noms::digit1)))(res)?;
    let end = end.map_or(start, |(_, v)| v.parse::<u32>().unwrap());
    Ok((res, (start, end)))
}

fn parse_position1(input: &str) -> noms::IResult<&str, u32> {
    let (res, start) = noms::digit1(input)?;
    let start = start.parse::<u32>().unwrap();
    Ok((res, start))
}

fn parse_sequence(input: &str) -> noms::IResult<&str, String> {
    let (res, seq) = noms::is_a("ATCGN")(input)?;
    Ok((res, seq.to_owned()))
}

fn parse_edit(input: &str) -> noms::IResult<&str, Edit> {
    let (res, edit) = noms::opt(noms::tag(">"))(input)?;
    if edit.is_some() {
        return Ok((res, Edit::Sub));
    }
    let (res, edit) = noms::opt(noms::tag("ins"))(input)?;
    if edit.is_some() {
        return Ok((res, Edit::Ins));
    };
    let (res, edit) = noms::opt(noms::tag("delins"))(input)?;
    if edit.is_some() {
        return Ok((res, Edit::Delins));
    }
    let (res, edit) = noms::opt(noms::tag("del"))(input)?;
    if edit.is_some() {
        return Ok((res, Edit::Del));
    }
    Err(noms::Err::Error(noms::Error::new(
        res,
        noms::ErrorKind::TagBits,
    )))
}

impl Variant {
    pub fn contig(&self) -> &str {
        &self.contig
    }

    pub fn start(&self) -> &u32 {
        &self.start
    }

    pub fn end(&self) -> &u32 {
        &self.end
    }

    pub fn edit(&self) -> &Edit {
        &self.edit
    }

    pub fn refseq(&self) -> Option<&str> {
        self.refseq.as_deref()
    }

    pub fn altseq(&self) -> Option<&str> {
        self.altseq.as_deref()
    }

    /// Length affected.
    pub fn affected_length(&self) -> u32 {
        match *self.edit() {
            Edit::Del => self.end - self.start + 1,
            Edit::Ins => self.altseq().map_or(0, |v| v.len() as u32),
            Edit::Sub => 1,
            Edit::Delins => u32::max(
                self.refseq().map_or(0, |v| v.len() as u32),
                self.altseq().map_or(0, |v| v.len() as u32),
            ),
            Edit::Identity => 0,
        }
    }

    /// Parse hgvs string into Variant
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use varlink::Variant;
    /// use varlink::Edit;
    ///
    /// let var = Variant::from_hgvs("1:12345A>G").unwrap();
    /// assert_eq!(var.contig(), "1");
    /// assert_eq!(var.start(), &12345u32);
    /// assert_eq!(var.end(), &12345u32);
    /// assert_eq!(var.edit(), &Edit::Sub);
    /// assert_eq!(var.refseq(), Some("A"));
    /// assert_eq!(var.altseq(), Some("G"));
    /// ```
    pub fn from_hgvs(input: &str) -> Result<Self, noms::Err<noms::Error<&str>>> {
        // Parse contig
        let (res, contig) = parse_contig(input)?;
        let (res, _) = noms::tag(":")(res)?;
        // Parse type
        let (res, _) = noms::opt(noms::tag("g."))(res)?;
        // Parse position
        let (res, (start, end)) = parse_position(res)?;
        // Parse ref sequence
        let (res, refseq) = noms::opt(parse_sequence)(res)?;
        // Parse edit
        let (res, edit) = parse_edit(res)?;
        // Parse alt sequence
        let (res, altseq) = noms::opt(parse_sequence)(res)?;
        let var = Variant {
            contig,
            start,
            end,
            edit,
            refseq,
            altseq,
        };
        if !matches!(var.edit, Edit::Del) && var.refseq.is_none() && var.altseq.is_none() {
            return Err(noms::Err::Error(noms::Error::new(
                res,
                noms::ErrorKind::Verify,
            )));
        };
        if !res.is_empty() {
            return Err(noms::Err::Error(noms::Error::new(
                res,
                noms::ErrorKind::NonEmpty,
            )));
        };
        Ok(var)
    }

    /// Parse vcf string into variant
    ///
    /// Vcf type variant comes in formats:
    ///
    /// - SNP: `chr1:12345A>G`
    /// - deletion: `chr1:12345AG>A`
    /// - insertion: `chr1:12345A>AG`
    /// - delins: `chr1:12345AGC>T`
    ///
    /// All with start position included, both ref and alt sequences are not empty.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use varlink::Variant;
    /// use varlink::Edit;
    ///
    /// let var = Variant::from_vcf("chr1:12345A>G").unwrap();
    /// assert_eq!(var.edit(), &Edit::Sub);
    /// let var = Variant::from_vcf("chr1:12345AG>A").unwrap();
    /// assert_eq!(var.edit(), &Edit::Del);
    /// let var = Variant::from_vcf("chr1:12345A>AG").unwrap();
    /// assert_eq!(var.edit(), &Edit::Ins);
    /// let var = Variant::from_vcf("chr1:12345AGC>T").unwrap();
    /// assert_eq!(var.start(), &12345);
    /// assert_eq!(var.end(), &12347);
    /// assert_eq!(var.edit(), &Edit::Delins);
    /// assert_eq!(var.contig(), "chr1");
    /// assert_eq!(var.refseq(), Some("AGC"));
    /// assert_eq!(var.altseq(), Some("T"));
    /// ```
    pub fn from_vcf(input: &str) -> Result<Self, noms::Err<noms::Error<&str>>> {
        // contig
        let (res, contig) = parse_contig(input)?;
        let (res, _) = noms::tag(":")(res)?;
        // position
        let (res, mut start) = parse_position1(res)?;
        // Parse ref sequence
        let (res, mut refseq) = parse_sequence(res)?;
        let (res, _) = noms::tag(">")(res)?;
        // Parse alt sequence
        let (res, mut altseq) = parse_sequence(res)?;
        if !res.is_empty() {
            return Err(noms::Err::Error(noms::Error::new(
                res,
                noms::ErrorKind::NonEmpty,
            )));
        };
        let var = if refseq == altseq {
            // Identity
            Variant {
                contig,
                start,
                end: start + refseq.len() as u32 - 1,
                edit: Edit::Identity,
                refseq: Some(refseq),
                altseq: Some(altseq),
            }
        } else if refseq.len() == 1 && altseq.len() == 1 {
            // Sub
            Variant {
                contig,
                start,
                end: start,
                edit: Edit::Sub,
                refseq: Some(refseq),
                altseq: Some(altseq),
            }
        } else if refseq.starts_with(&altseq) {
            // Deletion
            let len = altseq.len();
            let refseq = refseq.split_off(len);
            start += len as u32;
            let end = start + refseq.len() as u32 - 1;
            Variant {
                contig,
                start,
                end,
                edit: Edit::Del,
                refseq: Some(refseq),
                altseq: None,
            }
        } else if altseq.starts_with(&refseq) {
            // Insertion
            let len = refseq.len();
            let altseq = altseq.split_off(len);
            let end = start + 1;
            Variant {
                contig,
                start,
                end,
                edit: Edit::Ins,
                refseq: None,
                altseq: Some(altseq),
            }
        } else {
            let end = start + refseq.len() as u32 - 1;
            Variant {
                contig,
                start,
                end,
                edit: Edit::Delins,
                refseq: Some(refseq),
                altseq: Some(altseq),
            }
        };
        Ok(var)
    }

    /// Parse string to variant based on format given.
    ///
    /// ```rust
    /// use varlink::Variant;
    /// use varlink::VarFormat;
    ///
    /// let var1 = Variant::from("1:144852532G>GCCC", &VarFormat::Vcf).unwrap();
    /// let var2 = Variant::from("1:144852632TAA>T", &VarFormat::Vcf).unwrap();
    /// let var3 = Variant::from("1:g.144852532_144852533insCCC", &VarFormat::Hgvs).unwrap();
    /// let var4 = Variant::from("1:144852633_144852634del", &VarFormat::Hgvs).unwrap();
    /// assert_eq!(var1, var3);
    /// assert_eq!(
    ///     (var2.contig(), var2.start(), var2.end(), var2.edit()),
    ///     (var4.contig(), var4.start(), var4.end(), var4.edit())
    /// );
    /// ```
    pub fn from<'a>(input: &'a str, fmt: &Format) -> Result<Self, noms::Err<noms::Error<&'a str>>> {
        match fmt {
            Format::Hgvs => Self::from_hgvs(input),
            Format::Vcf => Self::from_vcf(input),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_position() {
        let pos_str = "12345_12346del";
        let (_, (start, end)) = parse_position(pos_str).unwrap();
        assert_eq!(start, 12345);
        assert_eq!(end, 12346);
        let pos_str = "12345A>G";
        let (_, (start, end)) = parse_position(pos_str).unwrap();
        assert_eq!(start, 12345);
        assert_eq!(end, 12345);
    }

    #[test]
    fn test_parse_snv() {
        let var_str = "1:12345A>G";
        let var = Variant::from_hgvs(var_str).unwrap();
        assert_eq!(
            Variant {
                contig: String::from("1"),
                start: 12345,
                end: 12345,
                edit: Edit::Sub,
                refseq: Some(String::from("A")),
                altseq: Some(String::from("G")),
            },
            var
        );
    }

    #[test]
    fn test_parse_insertion() {
        let var_str = "1:12345_12346insATCG";
        let var = Variant::from_hgvs(var_str).unwrap();
        assert_eq!(
            Variant {
                contig: String::from("1"),
                start: 12345,
                end: 12346,
                edit: Edit::Ins,
                refseq: None,
                altseq: Some(String::from("ATCG")),
            },
            var
        );
    }

    #[test]
    fn test_parse_deletion() {
        let var_str = "1:12345_12346del";
        let var = Variant::from_hgvs(var_str).unwrap();
        assert_eq!(
            Variant {
                contig: String::from("1"),
                start: 12345,
                end: 12346,
                edit: Edit::Del,
                refseq: None,
                altseq: None,
            },
            var
        );
    }
}
