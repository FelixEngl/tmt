use crate::dictionary::metadata::dict_meta_topic_matrix::META_DICT_ARRAY_LENTH;
use crate::dictionary::metadata::ex::MetadataEx;

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

