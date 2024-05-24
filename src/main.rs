use crate::topicmodel::vocabulary::Vocabulary;

mod topicmodel;
mod translate;
mod voting;
mod toolkit;


fn main() {

    let mut voc: Vocabulary<String> = Vocabulary::new();
    let _x = voc.add("Test".to_string());
    let _y = voc.add("Test2".to_string());
    println!("{:?}", voc.get_value(_x));
    println!("{:?}", voc.get_id(voc.get_value(_x).unwrap().as_ref()));
    println!("{:?}", voc.get_value(_y));
    println!("{:?}", voc.get_id(voc.get_value(_y).unwrap().as_ref()));
    println!("{}", voc.to_string());

}
