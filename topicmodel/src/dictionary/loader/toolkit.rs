macro_rules! replace_none_or_panic {
    ($opt: expr, $value: expr $(, $($tt:tt)*)?) => {
        if $opt.replace($value).is_some() {
            panic!($($($tt)*)?);
        }
    };
}


pub(super) use replace_none_or_panic;



#[cfg(test)]
pub mod for_test_only {
    use std::collections::{HashMap, HashSet};
    use std::sync::LazyLock;
    use convert_case::Case::Pascal;
    use convert_case::Casing;
    use itertools::Itertools;
    use regex::Regex;
    use strum::{VariantArray};
    use crate::dictionary::word_infos::Domain;

    // Source: https://github.com/haukex/de-en-dict/blob/main/src/js/abbreviations.json
    const DAT2: &'static str = r#"
    {
  "[Am.]": {
    "de": "Amerikanisches Englisch",
    "en": "American English"
  },
  "[Austr.]": {
    "de": "Australisches Englisch",
    "en": "Australian English"
  },
  "[Br.]": {
    "de": "Britisches Englisch",
    "en": "British English"
  },
  "[Can.]": {
    "de": "kanadisches Englisch",
    "en": "Canadian English"
  },
  "[Dt.]": {
    "de": "auf Deutschland beschr\u00e4nkter Sprachgebrauch",
    "en": "usage limited to Germany"
  },
  "[Ir.]": {
    "de": "Irisches Englisch",
    "en": "Irish English"
  },
  "[Jugendsprache]": {
    "de": "Sprachgebrauch unter Jugendlichen",
    "en": "usage when teenagers speak to each other"
  },
  "[J\u00e4gersprache]": {
    "de": "Fachjargon der J\u00e4ger",
    "en": "jargon used by hunters"
  },
  "[Kindersprache]": {
    "de": "Sprachgebrauch bei Gespr\u00e4chen mit oder unter Kindern",
    "en": "usage when speaking to children or when children speak among themselves"
  },
  "[Lie.]": {
    "de": "Sprachgebrauch in Liechtenstein",
    "en": "Liechtenstein usage; Liechtenstein idiom"
  },
  "[NZ]": {
    "de": "neuseel\u00e4ndisches Englisch",
    "en": "New Zealand English"
  },
  "[Sc.]": {
    "de": "Schottisches Englisch",
    "en": "Scottish English"
  },
  "[Schw.]": {
    "de": "Sprachgebrauch in der Schweiz; Helvetismus",
    "en": "Swiss usage; Swiss idiom"
  },
  "[Sprw.]": {
    "de": "Sprichwort",
    "en": "proverb"
  },
  "[S\u00fcdtirol]": {
    "de": "Sprachgebrauch in S\u00fcdtirol",
    "en": "South Tyrol usage; South Tyrol idiom"
  },
  "[adm.]": {
    "de": "Amtssprache; Verwaltung",
    "en": "official language; administration"
  },
  "[agr.]": {
    "de": "Landwirtschaft; Forstwirtschaft; Gartenbau",
    "en": "agriculture; forestry; gardening"
  },
  "[alt]": {
    "de": "alte deutsche Rechtschreibung",
    "en": "old German spelling"
  },
  "[altert\u00fcmlich]": {
    "de": "altert\u00fcmlich",
    "en": "archaic"
  },
  "[anat.]": {
    "de": "Anatomie",
    "en": "anatomy"
  },
  "[arch.]": {
    "de": "Architektur",
    "en": "architecture"
  },
  "[archaic]": {
    "de": "altert\u00fcmlich",
    "en": "archaic"
  },
  "[art]": {
    "de": "Kunst",
    "en": "arts"
  },
  "[astron.]": {
    "de": "Astronomie",
    "en": "astronomy"
  },
  "[auto]": {
    "de": "Kraftfahrzeugwesen; Stra\u00dfenverkehr",
    "en": "motoring; road traffic"
  },
  "[aviat.]": {
    "de": "Luftfahrt; Flugzeug",
    "en": "aviation; aircraft"
  },
  "[becoming dated]": {
    "de": "veraltend; teilweise als altmodisch empfunden",
    "en": "becoming dated; considered old-fashioned by some"
  },
  "[biochem.]": {
    "de": "Biochemie",
    "en": "biochemistry"
  },
  "[biol.]": {
    "de": "Biologie",
    "en": "biology"
  },
  "[bot.]": {
    "de": "Botanik; Pflanzen",
    "en": "botany; plants"
  },
  "[chem.]": {
    "de": "Chemie",
    "en": "chemistry"
  },
  "[children's speech]": {
    "de": "Sprachgebrauch bei Gespr\u00e4chen mit oder unter Kindern",
    "en": "usage when speaking to children or when children speak among themselves"
  },
  "[coll.]": {
    "de": "umgangssprachlich",
    "en": "colloquial"
  },
  "[comp.]": {
    "de": "Computerwesen; EDV; Informatik",
    "en": "computing; EDP; informatics"
  },
  "[constr.]": {
    "de": "Bauwesen",
    "en": "construction"
  },
  "[cook.]": {
    "de": "Speisen; Kochen; Essen; Gastronomie",
    "en": "dishes; cooking; eating; gastronomy "
  },
  "[dated]": {
    "de": "veraltet; altmodisch",
    "en": "dated; old-fashioned"
  },
  "[econ.]": {
    "de": "\u00d6konomie; Wirtschaft",
    "en": "economy"
  },
  "[electr.]": {
    "de": "Elektrotechnik, Elektronik",
    "en": "electrical engineering, electronics"
  },
  "[envir.]": {
    "de": "Umwelt; \u00d6kologie; Umweltschutz",
    "en": "environment; ecology; environmental protection "
  },
  "[euphem.]": {
    "de": "euphemistisch",
    "en": "euphemistic"
  },
  "[fig.]": {
    "de": "\u00fcbertragen; bildlich",
    "en": "figurative"
  },
  "[fin.]": {
    "de": "Finanzwesen",
    "en": "finance"
  },
  "[formal]": {
    "de": "gehoben",
    "en": "formal"
  },
  "[former name]": {
    "de": "durch eine neue Bezeichnung ersetzte, ehemals offizielle Bezeichnung",
    "en": "previously official term which was replaced by a new designation"
  },
  "[fr\u00fchere Bezeichnung]": {
    "de": "durch eine neue Bezeichnung ersetzte, ehemals offizielle Bezeichnung",
    "en": "previously official term which was replaced by a new designation"
  },
  "[geh.]": {
    "de": "gehoben",
    "en": "formal"
  },
  "[geogr.]": {
    "de": "Geografie",
    "en": "geography"
  },
  "[geol.]": {
    "de": "Geologie",
    "en": "geology"
  },
  "[hist.]": {
    "de": "Geschichte; Historisches",
    "en": "history"
  },
  "[humor.]": {
    "de": "humoristisch; scherzhaft",
    "en": "humorous; jocular"
  },
  "[hunters' parlance]": {
    "de": "Fachjargon der J\u00e4ger",
    "en": "jargon used by hunters"
  },
  "[iron.]": {
    "de": "ironisch",
    "en": "ironic"
  },
  "[jur.]": {
    "de": "Recht, Jura",
    "en": "law"
  },
  "[ling.]": {
    "de": "Linguistik; Sprachwissenschaft",
    "en": "linguistics"
  },
  "[lit.]": {
    "de": "Literatur; literarisch",
    "en": "literature; literarily"
  },
  "[mach.]": {
    "de": "Maschinenbau",
    "en": "machine construction"
  },
  "[math.]": {
    "de": "Mathematik",
    "en": "mathematics"
  },
  "[med.]": {
    "de": "Medizin",
    "en": "medicine"
  },
  "[meteo.]": {
    "de": "Meteorologie; Wetterkunde; Klimakunde",
    "en": "meteorology; climatology"
  },
  "[mil.]": {
    "de": "Milit\u00e4r; Waffenkunde",
    "en": "military; weaponry"
  },
  "[min.]": {
    "de": "Mineralogie; Bergbau",
    "en": "mineralogy; mining"
  },
  "[mus.]": {
    "de": "Musik",
    "en": "music"
  },
  "[myc.]": {
    "de": "Mykologie; Pilze",
    "en": "mycology; fungi"
  },
  "[naut.]": {
    "de": "Nautik; Schifffahrtskunde",
    "en": "nautical science; seafaring"
  },
  "[obs.]": {
    "de": "obsolet; nicht mehr in Gebrauch",
    "en": "obsolete; not longer used"
  },
  "[ornith.]": {
    "de": "Ornithologie; Vogelkunde",
    "en": "ornithology"
  },
  "[pej.]": {
    "de": "absch\u00e4tzig; abwertend; pejorativ",
    "en": "derogatory; pejorative"
  },
  "[pharm.]": {
    "de": "Pharmakologie; Arzneimittelkunde",
    "en": "pharmacology"
  },
  "[phil.]": {
    "de": "Philosophie",
    "en": "philosophy"
  },
  "[photo.]": {
    "de": "Fotografie",
    "en": "photography"
  },
  "[phys.]": {
    "de": "Physik",
    "en": "physics"
  },
  "[poet.]": {
    "de": "dichterisch: literarisch; poetisch",
    "en": "literary; poetic"
  },
  "[pol.]": {
    "de": "Politik",
    "en": "politics"
  },
  "[print]": {
    "de": "Druckwesen",
    "en": "printing"
  },
  "[prov.]": {
    "de": "Sprichwort",
    "en": "proverb"
  },
  "[psych.]": {
    "de": "Psychologie",
    "en": "psychology"
  },
  "[rare]": {
    "de": "selten verwendet",
    "en": "rarely used"
  },
  "[relig.]": {
    "de": "Religion",
    "en": "religion"
  },
  "[school]": {
    "de": "Schule; Bildung; Ausbildung",
    "en": "school; education; training "
  },
  "[sci.]": {
    "de": "Wissenschaft",
    "en": "science"
  },
  "[selten]": {
    "de": "selten verwendet",
    "en": "rarely used"
  },
  "[slang]": {
    "de": "Szenejargon; Dialekt; regionaler Sprachgebrauch; derbe Ausdrucksweise",
    "en": "slang; vernacular speech; regional language; coarse speech "
  },
  "[soc.]": {
    "de": "Soziologie; Gesellschaftsleben",
    "en": "sociology; social life "
  },
  "[sport]": {
    "de": "Sport",
    "en": "sports"
  },
  "[statist.]": {
    "de": "Statistik",
    "en": "statistics"
  },
  "[stud.]": {
    "de": "Studium (Hochschule)",
    "en": "studies (university)"
  },
  "[techn.]": {
    "de": "Technik",
    "en": "engineering"
  },
  "[telco.]": {
    "de": "Telekommunikation",
    "en": "telecommunications"
  },
  "[textil.]": {
    "de": "Textilindustrie; Bekleidung",
    "en": "textile industry; clothing"
  },
  "[transp.]": {
    "de": "Transport; Logistik",
    "en": "transportation; logistics"
  },
  "[ugs.]": {
    "de": "umgangssprachlich",
    "en": "colloquial"
  },
  "[veraltend]": {
    "de": "veraltend; teilweise als altmodisch empfunden",
    "en": "becoming dated; considered old-fashioned by some"
  },
  "[veraltet]": {
    "de": "veraltet; altmodisch",
    "en": "dated; old-fashioned"
  },
  "[vulg.]": {
    "de": "unfl\u00e4tig; obsz\u00f6n; vulg\u00e4r",
    "en": "vulgar; obscene"
  },
  "[youth speech]": {
    "de": "Sprachgebrauch unter Jugendlichen",
    "en": "usage when teenagers speak to each other"
  },
  "[zool.]": {
    "de": "Zoologie; Tiere",
    "en": "zoology; animals"
  },
  "[\u00d6s.]": {
    "de": "Sprachgebrauch in \u00d6sterreich; Austriazismus",
    "en": "Austrian usage; Austrian idiom"
  },
  "[\u00fcbtr.]": {
    "de": "\u00fcbertragen; bildlich",
    "en": "figurative"
  },
  "{adj}": {
    "de": "Adjektiv",
    "en": "adjective"
  },
  "{adv}": {
    "de": "Adverb; Adverbialphrase",
    "en": "adverb; adverbial phrase"
  },
  "{art}": {
    "de": "Artikel",
    "en": "article"
  },
  "{conj}": {
    "de": "Konjunktion",
    "en": "conjunction"
  },
  "{f}": {
    "de": "Substantiv, weiblich (die)",
    "en": "noun, feminine (die)"
  },
  "{interj}": {
    "de": "Interjektion; Ausruf",
    "en": "interjection"
  },
  "{m}": {
    "de": "Substantiv, m\u00e4nnlich (der)",
    "en": "noun, masculine (der)"
  },
  "{num}": {
    "de": "Numeral, Zahlwort",
    "en": "numeral"
  },
  "{n}": {
    "de": "Substantiv, s\u00e4chlich (das)",
    "en": "noun, neuter (das)"
  },
  "{pl}": {
    "de": "Substantiv, Plural (die)",
    "en": "noun, plural (die)"
  },
  "{ppron}": {
    "de": "Personalpronomen",
    "en": "personal pronoun"
  },
  "{pron}": {
    "de": "Pronomen",
    "en": "pronoun"
  },
  "{prp}": {
    "de": "Pr\u00e4position",
    "en": "preposition"
  },
  "{vi}": {
    "de": "Verb, intransitiv",
    "en": "verb, intransitive"
  },
  "{vr}": {
    "de": "Verb, reflexiv",
    "en": "verb, reflexive"
  },
  "{vt}": {
    "de": "Verb, transitiv",
    "en": "verb, transitive"
  },
  "{v}": {
    "de": "Verb, sonstig oder Verbalphrase",
    "en": "other verb, or verbal phrase"
  },
  "\u00ae": {
    "de": "Markenzeichen",
    "en": "trademark"
  }
}"#;

    #[test]
    fn produce(){
        let dat: HashMap<String, HashMap<String, String>> = serde_json::from_str(DAT2).unwrap();
        for v in dat.keys() {
            if let Ok(_) = v.parse::<Domain>() {
                println!("Domain: {v}")
            }

        }
    }


    #[derive(Copy, Clone, Debug, strum::Display, strum::EnumString, strum::VariantArray, Eq, PartialEq, Hash)]
    #[derive(serde::Serialize, serde::Deserialize)]
    #[derive(num_enum::TryFromPrimitive, num_enum::IntoPrimitive)]
    #[repr(u64)]
    pub enum Domain2 {
        #[strum(to_string = "bot.", serialize = "bot")]
        Bot = 0,
        #[strum(to_string = "hist.", serialize = "hist")]
        Hist = 1,
        #[strum(to_string = "phil.", serialize = "phil")]
        Phil = 2,
        #[strum(to_string = "chem.", serialize = "chem")]
        Chem = 3,
        #[strum(to_string = "arch.", serialize = "arch")]
        Arch = 4,
        #[strum(to_string = "transp.", serialize = "transp")]
        Transp = 5,
        #[strum(to_string = "min.", serialize = "min")]
        Min = 6,
        #[strum(to_string = "stud.", serialize = "stud")]
        Stud = 7,
        #[strum(to_string = "cook.", serialize = "cook")]
        Cook = 8,
        #[strum(to_string = "auto", serialize = "auto.")]
        Auto = 9,
        #[strum(to_string = "meteo.", serialize = "meteo")]
        Meteo = 10,
        #[strum(to_string = "art", serialize = "art.")]
        Art = 11,
        #[strum(to_string = "lit.", serialize = "lit")]
        Lit = 12,
        #[strum(to_string = "geogr.", serialize = "geogr")]
        Geogr = 13,
        #[strum(to_string = "ling.", serialize = "ling")]
        Ling = 14,
        #[strum(to_string = "telco.", serialize = "telco")]
        Telco = 15,
        #[strum(to_string = "pharm.", serialize = "pharm")]
        Pharm = 16,
        #[strum(to_string = "pol.", serialize = "pol")]
        Pol = 17,
        #[strum(to_string = "psych.", serialize = "psych")]
        Psych = 18,
        #[strum(to_string = "agr.", serialize = "agr")]
        Agr = 19,
        #[strum(to_string = "math.", serialize = "math")]
        Math = 20,
        #[strum(to_string = "statist.", serialize = "statist")]
        Statist = 21,
        #[strum(to_string = "mus.", serialize = "mus")]
        Mus = 22,
        #[strum(to_string = "sport", serialize = "sport.")]
        Sport = 23,
        #[strum(to_string = "anat.", serialize = "anat")]
        Anat = 24,
        #[strum(to_string = "astrol.", serialize = "astrol")]
        Astrol = 25,
        #[strum(to_string = "naut.", serialize = "naut")]
        Naut = 26,
        #[strum(to_string = "photo.", serialize = "photo")]
        Photo = 27,
        #[strum(to_string = "envir.", serialize = "envir")]
        Envir = 28,
        #[strum(to_string = "soc.", serialize = "soc")]
        Soc = 29,
        #[strum(to_string = "electr.", serialize = "electr")]
        Electr = 30,
        #[strum(to_string = "biol.", serialize = "biol")]
        Biol = 31,
        #[strum(to_string = "constr.", serialize = "constr")]
        Constr = 32,
        #[strum(to_string = "school", serialize = "school.")]
        School = 33,
        #[strum(to_string = "aviat.", serialize = "aviat")]
        Aviat = 34,
        #[strum(to_string = "fin.", serialize = "fin")]
        Fin = 35,
        #[strum(to_string = "mach.", serialize = "mach")]
        Mach = 36,
        #[strum(to_string = "archeol.", serialize = "archeol")]
        Archeol = 37,
        #[strum(to_string = "TV", serialize = "TV.")]
        Tv = 38,
        #[strum(to_string = "comp.", serialize = "comp")]
        Comp = 39,
        #[strum(to_string = "relig.", serialize = "relig")]
        Relig = 40,
        #[strum(to_string = "astron.", serialize = "astron")]
        Astron = 41,
        #[strum(to_string = "phys.", serialize = "phys")]
        Phys = 42,
        #[strum(to_string = "zool.", serialize = "zool")]
        Zool = 43,
        #[strum(to_string = "print", serialize = "print.")]
        Print = 44,
        #[strum(to_string = "econ.", serialize = "econ")]
        Econ = 45,
        #[strum(to_string = "textil.", serialize = "textil")]
        Textil = 46,
        #[strum(to_string = "biochem.", serialize = "biochem")]
        Biochem = 47,
        #[strum(to_string = "geol.", serialize = "geol")]
        Geol = 48,
        #[strum(to_string = "ornith.", serialize = "ornith")]
        Ornith = 49,
        #[strum(to_string = "med.", serialize = "med")]
        Med = 50,
        #[strum(to_string = "mil.", serialize = "mil")]
        Mil = 51,
        #[strum(to_string = "insur.", serialize = "insur")]
        Insur = 52,
    }

    const DAT: &str = r#"
acad.	Academic Disciplines / Wissenschaft	5666	1710
acc.	Accounting / Buchführung	1617	434
admin.	(Public) Administration / (Öffentliche) Verwaltung	3360	437
agr.	Agriculture, Aquaculture / Agrarwirtschaft, Land- und Gewässerbewirtschaftung	7217	306
anat.	Human Anatomy / Humananatomie	9839	553
archaeo.	Archaeology / Archäologie	2441	390
archi.	Architecture / Architektur	6450	512
armour	Historic Armour / Rüstungen, historische Schutzbekleidung	248	155
art	Art / Kunst	4918	1259
astron.	Astronomy / Astronomie	2905	665
astronau	Astronautics / Astronautik, Raumfahrt	553	365
audio	Audiology / Audiologie, Akustik	3564	224
automot.	Automotive Engineering / Automobil- und Fahrzeugtechnik	5729	526
aviat.	Aviation / Luftfahrt, Flugwesen	4537	506
bibl.	Biblical / Biblisch	1785	407
bike	Bicycle / Fahrrad	1052	546
biochem.	Biochemistry / Biochemie	7270	354
biol.	Biology / Biologie	16424	720
biotech.	Biotechnology / Biotechnologie	481	306
bot.	Botany, Plants / Botanik, Pflanzen	57838	428
brew	Brewing / Brauwesen	495	170
chem.	Chemistry / Chemie	14655	582
climbing	Climbing, Mountaineering / Bergsteigerei	522	272
cloth.	Clothing, Fashion / Bekleidung, Mode	9150	474
comics	Comics and Animated Cartoons / Comics und Zeichentrickfilme	413	371
comm.	Commerce / Handel	10074	571
comp.	Computer Sciences / Informatik, IT	12238	1728
constr.	Construction / Bauwesen	7740	356
cosmet.	Cosmetics & Body Care / Kosmetik und Körperpflege	1688	272
curr.	Currencies / Währungen	932	279
dance	Dance / Tanz	923	405
dent.	Dental Medicine / Zahnmedizin	5082	137
drugs	Drugs / Drogen	1075	288
ecol.	Ecology, Environment / Ökologie, Umwelt	5939	762
econ.	Economy / Wirtschaft, Ökonomie	11561	970
educ.	Education / Ausbildung	8405	1280
electr.	Electrical Engin., Electronics / Elektrotechnik, Elektronik	13848	737
engin.	Engineering / Ingenieurwissenschaften	2926	1005
entom.	Entomology / Entomologie, Insektenkunde	14402	114
equest.	Equestrianism, Horses / Reitsport, Pferde	1443	136
esot.	Esotericism / Esoterik	432	250
ethn.	Ethnology / Ethnologie	2881	293
EU	European Union / Europäische Union	1541	706
F	Fiction: Names and Titles in Literature, Film, TV, Arts / Fiktion: Namen und Titel in Literatur, Film, TV, Kunst	14905	843
film	Film / Film	9570	1346
fin.	Finance / Finanzwesen	10018	608
FireResc	Firefighting & Rescue / Feuerwehr & Rettungsdienst	1290	141
fish	Ichthyology, fish, fishing / Fischkunde, Fischen, Angelsport	11242	115
FoodInd.	Foodstuffs Industry / Lebensmittelindustrie	3230	321
for.	Forestry / Forstwissenschaft, Forstwirtschaft	1520	182
furn.	Furniture / Möbel	1988	243
games	Games / Spiele	2362	958
gastr.	Gastronomy, Cooking / Gastronomie, Kochen	18747	690
geogr.	Geography / Geografie	13396	918
geol.	Geology / Geologie	7909	326
herald.	Heraldry / Heraldik	262	117
hist.	History / Historische Begriffe, Geschichte	16815	1412
hort.	Horticulture / Gartenbau	2569	255
hunting	Hunting / Jagd	1114	136
hydro.	Hydrology & Hydrogeology / Hydrologie & Hydrogeologie	2325	116
idiom	Idiom / Idiom, Redewendung	6725	708
ind.	Industry / Industrie	4340	533
insur.	Insurance / Versicherungswesen	1988	179
Internet	Internet / Internet	1894	1526
jobs	Jobs, Employment Market / Berufe, Arbeitsmarkt	15646	604
journ.	Journalism / Journalismus	1840	468
law	Law / Jura, Rechtswesen	16939	619
libr.	Library Science / Bibliothekswissenschaft	590	202
ling.	Linguistics / Linguistik, Sprachwissenschaft	7235	1495
lit.	Literature / Literatur	10481	1305
market.	Marketing, Advertising / Marketing, Werbung, Vertrieb und Handelswesen	1799	569
material	Materials Science / Materialwissenschaft, Werkstoffkunde	3376	307
math.	Mathematics / Mathematik	9257	869
med.	Medicine / Medizin	54873	885
MedTech.	Medical Engineering & Imaging / Medizintechnik	9370	273
meteo.	Meteorology / Meteorologie	3856	243
mil.	Military / Militärwesen	10882	439
mineral.	Mineralogy / Mineralogie	6409	161
mining	Mining & Drilling / Bergbau & Bohrtechnik	2155	118
mus.	Music / Musik	11901	1816
mycol.	Mycology / Mykologie, Pilze	4857	121
myth.	Mythology / Mythologie	1888	541
name	Names of Persons / Namenkunde (nur Personennamen)	408	235
naut.	Nautical Science / Nautik, Schifffahrtskunde	6102	230
neol.	Neologisms / Neologismen (Wortneubildungen)	279	194
nucl.	Nuclear Engineering / Nukleartechnik	698	194
oenol.	Oenology / Önologie, Lehre vom Wein	1587	130
optics	Optics / Optik	1807	186
orn.	Ornithology / Ornithologie, Vogelkunde	29409	157
pharm.	Pharmacy / Pharmazie	6206	309
philat.	Philately / Philatelie, Briefmarkenkunde	238	65
philos.	Philosophy / Philosophie	4102	1007
phonet.	Phonetics / Phonetik	482	436
photo.	Photography / Fotografie	2530	859
phys.	Physics / Physik	9634	853
pol.	Politics / Politik	13500	1048
print	Print, Typography, Layout / Druck, Typografie, Layout	2104	294
proverb	Proverb / Sprichwort	1192	649
psych.	Psychology / Psychologie	7264	965
publ.	Publishing / Verlagswesen	2123	276
QM	Quality Management / Qualitätsmanagement	2269	294
quote	Quotation / Zitat	712	494
RadioTV	Radio and Television / Radio und Fernsehen	4069	573
rail	Rail / Eisenbahn	3098	230
RealEst.	Real Estate / Immobilien	1883	247
relig.	Religion / Religion	15423	718
rhet.	Rhetoric / Rhetorik	246	427
sociol.	Sociology / Soziologie	3993	580
spec.	Specialized Term / Fachsprachlicher Ausdruck	5444	389
sports	Sports / Sport	12703	993
stat.	Statistics / Statistik	1677	375
stocks	Stock Exchange / Börsenwesen	2043	272
T	Taxonomic terms for animals, plants and fungi (incl. varieties and breeds) / Taxonomische Bezeichnungen für Tiere, Pflanzen und Pilze (inkl. Zuchtformen und Rassen)	134957	16
tech.	Technology / Technik	34014	1344
telecom.	Telecommunications / Telekommunikation	2618	560
textil.	Textiles, Textile Industry / Textilien, Textilindustrie	2701	168
theatre	Theatre / Theater	2842	520
tools	Tools / Werkzeuge	5133	523
toys	Toys / Spielzeug	649	237
traffic	Traffic / Verkehrswesen	2704	338
transp.	Transportation (Land Transport) / Transportwesen (Landtransport)	3369	360
travel	Travel Industry / Touristik	2293	558
TrVocab.	Travellers vocabulary / Reise-Wortschatz	1445	724
unit	Units, Measures, Weights / Einheiten, Maße, Gewichte	1011	452
urban	Urban Planning / Urbanistik, Stadtplanung	1072	251
UWH	UNESCO World Heritage / UNESCO-Welterbe	126	281
VetMed.	Veterinary Medicine / Veterinärmedizin	3056	176
watches	Watches, Clocks / Uhren	520	215
weapons	Weapons / Waffen	2858	415
zool.	Zoology, Animals / Zoologie, Tierkunde	39299	688
        "#;

    #[test]
    fn generate_direction_data() {

        let mut words_with_comment = HashMap::new();
        for line in DAT.lines() {
            let line = line.trim();
            if line.trim().is_empty() {
                continue
            }
            let mut split = line.split("\t");
            let first = split.next().unwrap();
            let second = split.next().unwrap();
            words_with_comment.insert(create_enum_name(first), (Some(second.to_string()), first.to_string(), vec![]));
        }

        for value in Domain2::VARIANTS.into_iter() {
            let repr = value.to_string();
            match repr.as_str() {
                "archeol." => {
                    words_with_comment.get_mut("Archaeo").unwrap().2.push(repr)
                }
                "arch." => {
                    words_with_comment.get_mut("Archi").unwrap().2.push(repr)
                }
                "astrol." => {
                    words_with_comment.entry(create_enum_name(&repr)).or_insert((Some("Astrology / Astrologie".to_string()), repr, vec![]));
                }
                "auto" => {
                    words_with_comment.get_mut("Automot").unwrap().2.push(repr)
                }
                "cook." => {
                    words_with_comment.entry(create_enum_name(&repr)).or_insert((Some("Cooking".to_string()), repr, vec![]));
                }
                "envir." => {
                    words_with_comment.get_mut("Ecol").unwrap().2.push(repr)
                }
                "mach." => {
                    words_with_comment.entry(create_enum_name(&repr)).or_insert((Some("Machines".to_string()), repr, vec![]));
                }
                "min." => {
                    words_with_comment.get_mut("Mining").unwrap().2.push(repr)
                }
                "ornith." => {
                    words_with_comment.get_mut("Orn").unwrap().2.push(repr)
                }
                "phil." => {
                    words_with_comment.get_mut("Philos").unwrap().2.push(repr)
                }
                "school" => {
                    words_with_comment.entry(create_enum_name(&repr)).or_insert((Some("School/Schule".to_string()), repr, vec![]));
                }
                "soc." => {
                    words_with_comment.get_mut("Sociol").unwrap().2.push(repr)
                }
                "sport" => {
                    words_with_comment.get_mut("Sports").unwrap().2.push(repr)
                }
                "statist." => {
                    words_with_comment.get_mut("Stat").unwrap().2.push(repr)
                }
                "stud." => {
                    words_with_comment.entry(create_enum_name(&repr)).or_insert((Some("Studium".to_string()), repr, vec![]));
                }
                "telco." => {
                    words_with_comment.get_mut("Telecom").unwrap().2.push(repr)
                }
                "TV" => {
                    words_with_comment.get_mut("RadioTv").unwrap().2.push(repr)
                }
                _ => {
                    words_with_comment.entry(create_enum_name(&repr)).or_insert((None, repr, vec![]));
                }
            }

        }

        words_with_comment.get_mut("Idiom").unwrap().2.push("Redewendung".to_string());
        let mut v = words_with_comment.into_iter().collect_vec();
        v.sort_by_key(|value| value.0.clone());

        let e = create_enum_definition(
            "Domain",
            v.into_iter().map(|value| {
                EnumEntry {
                    name_base: value.1.1,
                    comment: value.1.0,
                    synonyms: value.1.2
                }
            }),
            true,
            None,
            true
        );

        println!("{e}");
    }

    fn create_enum_name(name: &str) -> String {
        static REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new("[^a-zA-Z0-9]").unwrap());
        let name = name.trim();
        let result = REGEX.replace_all(name, "_").to_case(Pascal);
        if result.starts_with(|c| matches!(c, 'a'..='z' | 'A'..='Z')) {
            result
        } else {
            const FIXED_PREFIX: &str = "ZZZ_FIXED_";
            let mut s = String::with_capacity(result.len() + FIXED_PREFIX.len());
            s.push_str(FIXED_PREFIX);
            s.push_str(&result);
            s
        }
    }

    struct EnumEntry<V> {
        comment: Option<V>,
        name_base: V,
        synonyms: Vec<V>
    }

    impl<V> From<V> for EnumEntry<V> {
        fn from(value: V) -> Self {
            Self {
                comment: None,
                name_base: value,
                synonyms: Vec::new()
            }
        }
    }

    fn create_enum_definition<I: IntoIterator<Item = EnumEntry<V>>, V: AsRef<str>>(
        name: &str,
        iter: I,
        with_u64_repr: bool,
        base_value: Option<usize>,
        serialize_with_end_dot: bool
    ) -> String {
        let base_value = base_value.unwrap_or(0);
        use std::fmt::Write;
        let mut s = String::new();
        write!(s, "#[derive(Copy, Clone, Debug, strum::Display, strum::EnumString, Eq, PartialEq, Hash)]\n").unwrap();
        write!(s, "#[derive(serde::Serialize, serde::Deserialize)]\n").unwrap();
        if with_u64_repr {
            write!(s, "#[derive(num_enum::TryFromPrimitive, num_enum::IntoPrimitive)]\n").unwrap();
            write!(s, "#[repr(u64)]\n").unwrap();
        }
        write!(s, "pub enum {} {{\n", name.to_case(Pascal)).unwrap();
        for (i, EnumEntry {
            synonyms,
            comment,
            name_base
        }) in iter.into_iter().enumerate() {
            let original_name = name_base.as_ref().trim();
            let enum_name = create_enum_name(original_name);
            if let Some(comment) = comment {
                for l in comment.as_ref().lines() {
                    write!(s, "    /// {}\n", l).unwrap();
                }
            } else {
                write!(s, "    /// NEEDS MAPPING\n").unwrap();
            }
            if serialize_with_end_dot {
                let mut names = HashSet::new();
                for syn in synonyms.into_iter() {
                    let s = syn.as_ref().trim();
                    names.insert(s.to_string());
                    names.insert(s.to_lowercase());
                    names.insert(s.to_uppercase());
                    if s.ends_with(".") {
                        let x = s.trim_end_matches(".");
                        names.insert(x.to_string());
                        names.insert(x.to_lowercase());
                        names.insert(x.to_uppercase());
                    } else {
                        let x = format!("{s}.");
                        names.insert(x.to_string());
                        names.insert(x.to_lowercase());
                        names.insert(x.to_uppercase());
                    }
                }
                if original_name != original_name.to_uppercase() {
                    names.insert(original_name.to_uppercase());
                }
                if original_name != original_name.to_lowercase() {
                    names.insert(original_name.to_lowercase());
                }
                if original_name.ends_with(".") {
                    let dot = original_name.trim_end_matches(".");
                    names.insert(dot.to_string());
                    names.insert(dot.to_uppercase());
                    names.insert(dot.to_lowercase());
                    write!(s, "    #[strum(to_string = \"{}\", {})]\n", original_name, names.into_iter().map(|value| format!("serialize = \"{value}\"")).join(", ")).unwrap();
                } else {
                    let u = format!("{}.", original_name);
                    names.insert(u.to_string());
                    names.insert(u.to_uppercase());
                    names.insert(u.to_lowercase());
                    write!(s, "    #[strum(to_string = \"{}\", {})]\n", original_name, names.into_iter().map(|value| format!("serialize = \"{value}\"")).join(", ")).unwrap();
                }
            } else {
                let names = synonyms.into_iter().map(|value| value.as_ref().trim().to_string()).collect::<HashSet<_>>();
                write!(s, "    #[strum(to_string = \"{}\", {})]\n", original_name, names.into_iter().map(|value| format!("serialize = \"{value}\"")).join(", ")).unwrap();
            }

            if with_u64_repr {
                write!(s, "    {} = {},\n", enum_name, i + base_value).unwrap();
            } else {
                write!(s, "    {},\n", enum_name).unwrap();
            }
        }
        write!(s, "}}").unwrap();
        if with_u64_repr {
            write!(s, "\n\nimpl tinyset::set64::Fits64 for {} {{\n", name.to_case(Pascal)).unwrap();
            write!(s, "    #[inline(always)]\n").unwrap();
            write!(s, "    unsafe fn from_u64(x: u64) -> Self {{\n").unwrap();
            write!(s, "        {}::try_from(x).unwrap()\n", name.to_case(Pascal)).unwrap();
            write!(s, "    }}\n").unwrap();
            write!(s, "    #[inline(always)]\n").unwrap();
            write!(s, "    fn to_u64(self) -> u64 {{\n").unwrap();
            write!(s, "        self.into()\n").unwrap();
            write!(s, "    }}\n").unwrap();
            write!(s, "}}").unwrap();
        }
        s
    }




}