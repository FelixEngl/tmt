use crate::topicmodel::vocabulary::Vocabulary;

mod topicmodel;


fn main() {

    let mut voc: Vocabulary<String> = Vocabulary::new();
    let _x = voc.add("Test".to_string());
    let _y = voc.add("Test2".to_string());
    println!("{:?}", voc.get_word(_x));
    println!("{:?}", voc.get_word_id(voc.get_word(_x).unwrap()));
    println!("{:?}", voc.get_word(_y));
    println!("{:?}", voc.get_word_id(voc.get_word(_y).unwrap()));
    println!("{}", voc.to_string());

}
