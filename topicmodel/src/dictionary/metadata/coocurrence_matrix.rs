use itertools::Itertools;
use crate::dictionary::metadata::dict_meta_topic_matrix::META_DICT_ARRAY_LENTH;
use crate::dictionary::metadata::ex::{DictMetaCount, MetadataEx};

pub fn co_occurrence_count_for<'a, I: IntoIterator<Item=&'a MetadataEx> + 'a>(m: I) -> [[usize; META_DICT_ARRAY_LENTH]; META_DICT_ARRAY_LENTH] {
    let mut value = [[0usize; META_DICT_ARRAY_LENTH]; META_DICT_ARRAY_LENTH];
    for meta in m.into_iter() {
        let count = meta.domain_count();
        for (i_outer, &ct_outer) in count.iter().enumerate() {
            if ct_outer == 0 {
                continue
            }
            for (i_inner, &ct_inner) in count.iter().enumerate() {
                if ct_inner == 0 {
                    continue;
                }
                value[i_outer][i_inner] += 1;
            }
        }
    }
    value
}



pub fn co_occurences_direct_a_to_b<'a, I: IntoIterator<Item=(&'a MetadataEx, &'a MetadataEx)> + 'a>(m: I) -> DictMetaCount {
    let mut value = [0u64; META_DICT_ARRAY_LENTH];
    for (a, b) in m.into_iter() {
        let a_count = a.domain_count();
        let b_count = b.domain_count();
        for (i, (ct_a, ct_b)) in a_count.into_iter().zip_eq(b_count.into_iter()).enumerate() {
            if ct_a == 0 && ct_b == 0 {
                continue
            }
            value[i] += 1
        }
    }
    DictMetaCount::new(value)
}

pub fn co_occurences_a_to_b<'a, const COUNT: bool, I: IntoIterator<Item=(&'a MetadataEx, &'a MetadataEx)> + 'a>(m: I) -> [[usize; META_DICT_ARRAY_LENTH]; META_DICT_ARRAY_LENTH] {
    let mut value = [[0usize; META_DICT_ARRAY_LENTH]; META_DICT_ARRAY_LENTH];
    for (a, b) in m.into_iter() {
        let a_count = a.domain_count();
        let b_count = b.domain_count();
        for (i_out, ct_a) in a_count.into_iter().enumerate() {
            if ct_a == 0 {
                continue
            }
            for (i_in, &ct_b) in b_count.iter().enumerate() {
                if ct_b == 0 {
                    continue
                }
                value[i_out][i_in] += if COUNT {
                    1usize
                } else {
                    ct_a as usize + ct_b as usize
                };
            }
        }
    }
    value
}