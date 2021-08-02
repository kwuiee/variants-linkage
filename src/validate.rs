use bam::record::AlignmentEntry;
use bam::record::Record;

use crate::{Edit, Variant};

#[derive(Debug, Default)]
pub struct ValidateOptions {
    // If *merge* variants exists, do not consider record as support.
    merge: bool,
}

impl ValidateOptions {
    pub fn set_merge(&mut self, merge: bool) {
        self.merge = merge;
    }
}

fn logical_merge(e1: &Edit, e2: &Edit, merge: bool) -> bool {
    if e1.is_identity() || e2.is_identity() {
        false
    } else if e1.is_delins() && e2.is_sub() {
        merge
    } else if e1.is_delins() || e2.is_delins() {
        true
    } else if (e1.is_del() ^ e2.is_del())
        || (e1.is_ins() ^ e2.is_ins())
        || e1.is_sub()
        || e2.is_sub()
    {
        merge
    } else {
        true
    }
}

pub trait VariantValidate {
    fn validate(&self, variant: &Variant, option: &ValidateOptions) -> Option<bool>;
}

impl VariantValidate for Record {
    /// Validate record supportion for variant.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use bam::record::Record;
    /// use varlink::{Variant, VariantValidate, ValidateOptions};
    ///
    /// let var = Variant::parse("chr1:123456Adel").unwrap();
    /// let options = ValidateOptions::default();
    /// let record = Record::new();
    ///
    /// assert!(record.validate(&var, &options).is_none());
    /// ```
    ///
    /// ## Warn
    ///
    /// Crate `bam` bam reader reading alignemnt with 0-based position, while variant is 1-based.
    /// So alignment `+1` or variant `-1` is necessary in some places.
    #[allow(clippy::blocks_in_if_conditions)]
    fn validate(&self, variant: &Variant, options: &ValidateOptions) -> Option<bool> {
        // Unmapped read or out of region.
        if (!self.flag().is_mapped())
            || (self.start() + 1) as u32 > *variant.end()
            || (self.calculate_end() as u32) < *variant.start()
        {
            return None;
        }

        let mut iter = if let Ok(v) = self.alignment_entries() {
            v.skip_while(|i| {
                i.ref_pos() < Some(variant.start() - 1 - (!variant.edit().is_ins() as u32))
            })
        } else {
            return None;
        };

        // If nt before start position is insertion, deletion or mismatch.
        let mut next: Option<AlignmentEntry> = if let Some(v) = iter.next() {
            Some(v)
        } else {
            return None;
        };
        if next.as_ref().map_or(false, |v| {
            let curredit = if v.is_insertion() {
                Edit::Ins
            } else if v.is_deletion() {
                Edit::Del
            } else if !v.is_seq_match() {
                Edit::Sub
            } else {
                Edit::Identity
            };
            logical_merge(&curredit, variant.edit(), options.merge)
        }) {
            return Some(false);
        };

        let refdef = if variant.edit().is_del() {
            unsafe { String::from_utf8_unchecked(vec![b'N'; variant.affected_length() as usize]) }
        } else {
            String::from("")
        };

        let mut refseq = variant.refseq().unwrap_or(&refdef).as_bytes().iter();
        let mut refnt = refseq.next();
        let mut altseq = variant.altseq().unwrap_or("").as_bytes().iter();
        let mut altnt = altseq.next();

        // In case both ref and alt are empty.
        if refnt.is_none() && altnt.is_none() && !variant.edit().is_del() {
            return Some(true);
        };

        next = iter.next();
        loop {
            let curr = match next {
                Some(v) => v,
                // Variant validated, but no more nt for next record position.
                None => break Some(refnt.is_none() && altnt.is_none()),
            };
            let curredit = if curr.is_insertion() {
                Edit::Ins
            } else if curr.is_deletion() {
                Edit::Del
            } else if !curr.is_seq_match() {
                Edit::Sub
            } else {
                Edit::Identity
            };

            if refnt.is_none()
                && altnt.is_none()
                && logical_merge(&curredit, variant.edit(), options.merge)
            {
                // Variant validated, but next record position is deletion, insertion or mismatch.
                break Some(false);
            } else if refnt.is_none() && altnt.is_none() {
                // Variant validated.
                break Some(true);
            };

            next = iter.next();

            if curr.is_insertion() && curr.record_nt().as_ref() == altnt {
                // Insertion consumes one variant alt nt.
                altnt = altseq.next();
            } else if curr.is_insertion() {
                break Some(false);
            } else if curr.is_deletion() && curr.ref_nt().is_some() {
                // Deletion consumes one variant ref nt.
                refnt = refseq.next();
            } else if curr.is_deletion() {
                break Some(false);
            } else if curr.record_nt().as_ref() == altnt && curr.ref_nt().as_ref() == refnt {
                // Match or mismatch consumes a variant ref and alt pair nts.
                refnt = refseq.next();
                altnt = altseq.next();
            } else {
                break Some(false);
            };
        }
    }
}
