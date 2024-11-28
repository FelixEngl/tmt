use num_enum::{IntoPrimitive, TryFromPrimitive};
use pyo3::{pyclass, pymethods};
use serde::{Deserialize, Serialize};
use strum::{Display, EnumCount, EnumIter, EnumString, IntoStaticStr, VariantArray};
use tinyset::Fits64;
use ldatranslate_toolkit::register_python;
use crate::dictionary::metadata::dict_meta_topic_matrix::{DomainModelIndex, NotAIndexFor};
use crate::dictionary::metadata::ex::impl_try_from_as_unpack;

register_python! {
    enum Domain;
}

#[cfg_attr(feature="gen_python_api", pyo3_stub_gen::derive::gen_stub_pyclass_enum)]
#[pyclass(eq, eq_int, hash, frozen)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
#[derive(Display, EnumString, IntoStaticStr, EnumCount, EnumIter, VariantArray)]
#[derive(TryFromPrimitive, IntoPrimitive, Serialize, Deserialize)]
#[repr(u16)]
pub enum Domain {
    /// Academic Disciplines / Wissenschaft
    #[strum(
        to_string = "acad.",
        serialize = "ACAD.",
        serialize = "ACAD",
        serialize = "acad",
        serialize = "academic",
        serialize = "academia",
    )]
    Acad = 0,
    /// Accounting / Buchführung
    #[strum(
        to_string = "acc.",
        serialize = "ACC",
        serialize = "ACC.",
        serialize = "acc",
        serialize = "accounting",
    )]
    Acc = 1,
    /// (Public) Administration / (Öffentliche) Verwaltung
    #[strum(
        to_string = "admin.",
        serialize = "ADMIN.",
        serialize = "ADMIN",
        serialize = "admin",
    )]
    Admin = 2,
    /// Agriculture, Aquaculture / Agrarwirtschaft, Land- und Gewässerbewirtschaftung
    #[strum(
        to_string = "agr.",
        serialize = "AGR.",
        serialize = "agr",
        serialize = "AGR",
        serialize = "agriculture",
    )]
    Agr = 3,
    /// Human Anatomy / Humananatomie
    #[strum(
        to_string = "anat.",
        serialize = "ANAT",
        serialize = "ANAT.",
        serialize = "anat",
        serialize = "Anatomy",
        serialize = "anatomy",
    )]
    Anat = 4,
    /// Archaeology / Archäologie
    #[strum(
        to_string = "archaeo.",
        serialize = "archeol",
        serialize = "ARCHEOL.",
        serialize = "ARCHAEO.",
        serialize = "ARCHAEO",
        serialize = "ARCHEOL",
        serialize = "archaeo",
        serialize = "archeol.",
        serialize = "archaeology",
    )]
    Archaeo = 5,
    /// Architecture / Architektur
    #[strum(
        to_string = "archi.",
        serialize = "ARCHI",
        serialize = "arch",
        serialize = "ARCH.",
        serialize = "archi",
        serialize = "ARCHI.",
        serialize = "arch.",
        serialize = "ARCH",
        serialize = "architecture",
    )]
    Archi = 6,
    /// Historic Armour / Rüstungen, historische Schutzbekleidung
    #[strum(
        to_string = "armour",
        serialize = "ARMOUR",
        serialize = "armour.",
        serialize = "ARMOUR."
    )]
    Armour = 7,
    /// Art / Kunst
    #[strum(
        to_string = "art",
        serialize = "art.",
        serialize = "ART.",
        serialize = "ART",
        serialize = "arts",
        serialize = "theater",
        serialize = "acting",
        serialize = "drama",
        serialize = "dramaturgy",
    )]
    Art = 8,
    /// Astrology / Astrologie
    #[strum(
        to_string = "astrol.",
        serialize = "ASTROL",
        serialize = "ASTROL.",
        serialize = "astrol",
        serialize = "astrology",
    )]
    Astrol = 9,
    /// Astronomy / Astronomie
    #[strum(
        to_string = "astron.",
        serialize = "ASTRON.",
        serialize = "astron",
        serialize = "ASTRON",
        serialize = "astronomy",
        serialize = "cosmology",
        serialize = "space-science",
        serialize = "space-sciences",
    )]
    Astron = 10,
    /// Astronautics / Astronautik, Raumfahrt
    #[strum(
        to_string = "astronau",
        serialize = "astronau.",
        serialize = "ASTRONAU.",
        serialize = "astronautics",
        serialize = "ASTRONAU",
        serialize = "aerospace",
    )]
    Astronau = 11,
    /// Audiology / Audiologie, Akustik
    #[strum(
        to_string = "audio",
        serialize = "AUDIO.",
        serialize = "AUDIO",
        serialize = "audio.",
    )]
    Audio = 12,
    /// Automotive Engineering / Automobil- und Fahrzeugtechnik
    #[strum(
        to_string = "automot.",
        serialize = "AUTO.",
        serialize = "AUTO",
        serialize = "auto",
        serialize = "AUTOMOT",
        serialize = "AUTOMOT.",
        serialize = "automot",
        serialize = "auto.",
        serialize = "automotive",
        serialize = "automobile",
    )]
    Automot = 13,
    /// Aviation / Luftfahrt, Flugwesen
    #[strum(
        to_string = "aviat.",
        serialize = "AVIAT.",
        serialize = "aviat",
        serialize = "AVIAT",
        serialize = "aviation",
        serialize = "aerodynamics",
        serialize = "aeronautics",
    )]
    Aviat = 14,
    /// Biblical / Biblisch
    #[strum(
        to_string = "bibl.",
        serialize = "bibl",
        serialize = "BIBL.",
        serialize = "BIBL",
        serialize = "biblical",
    )]
    Bibl = 15,
    /// Bicycle / Fahrrad
    #[strum(
        to_string = "bike",
        serialize = "BIKE",
        serialize = "bike.",
        serialize = "BIKE.",
    )]
    Bike = 16,
    /// Biochemistry / Biochemie
    #[strum(
        to_string = "biochem.",
        serialize = "BIOCHEM.",
        serialize = "biochem",
        serialize = "BIOCHEM",
        serialize = "biochemistry",
    )]
    Biochem = 17,
    /// Biology / Biologie
    #[strum(
        to_string = "biol.",
        serialize = "BIOL.",
        serialize = "BIOL",
        serialize = "biol",
        serialize = "sociobiology",
        serialize = "biology",
        serialize = "genetics",
        serialize = "microbiology",
        serialize = "neurobiology",
        serialize = "paleobiology",
    )]
    Biol = 18,
    /// Biotechnology / Biotechnologie
    #[strum(
        to_string = "biotech.",
        serialize = "biotech",
        serialize = "BIOTECH.",
        serialize = "BIOTECH",
        serialize = "biotechnology",
    )]
    Biotech = 19,
    /// Botany, Plants / Botanik, Pflanzen
    #[strum(
        to_string = "bot.",
        serialize = "bot",
        serialize = "BOT.",
        serialize = "BOT",
    )]
    #[strum(serialize = "botany"
    )]
    Bot = 20,
    /// Brewing / Brauwesen
    #[strum(
        to_string = "brew",
        serialize = "BREW.",
        serialize = "brew.",
        serialize = "BREW",
        serialize = "brewing",
    )]
    Brew = 21,
    /// Chemistry / Chemie
    #[strum(
        to_string = "chem.",
        serialize = "chem",
        serialize = "CHEM",
        serialize = "CHEM.",
        serialize = "chemistry",
        serialize = "immunochemistry",
        serialize = "organic-chemistry",
        serialize = "inorganic-chemistry",
        serialize = "petrochemistry",
    )]
    Chem = 22,
    /// Climbing, Mountaineering / Bergsteigerei
    #[strum(
        to_string = "climbing",
        serialize = "CLIMBING.",
        serialize = "CLIMBING",
        serialize = "climbing.",
        serialize = "electrochemistry",
    )]
    Climbing = 23,
    /// Clothing, Fashion / Bekleidung, Mode
    #[strum(
        to_string = "cloth.",
        serialize = "cloth",
        serialize = "CLOTH",
        serialize = "CLOTH.",
        serialize = "clothing",
        serialize = "dressmaking",
    )]
    Cloth = 24,
    /// Comics and Animated Cartoons / Comics und Zeichentrickfilme
    #[strum(
        to_string = "comics",
        serialize = "comics.",
        serialize = "COMICS",
        serialize = "COMICS.",
    )]
    Comics = 25,
    /// Commerce / Handel
    #[strum(
        to_string = "comm.",
        serialize = "comm",
        serialize = "COMM",
        serialize = "COMM.",
        serialize = "commerce",
        serialize = "commercial",
    )]
    Comm = 26,
    /// Computer Sciences / Informatik, IT
    #[strum(
        to_string = "comp.",
        serialize = "COMP.",
        serialize = "COMP",
        serialize = "comp",
        serialize = "cryptography",
        serialize = "MIDI",
        serialize = "Linux",
        serialize = "programming",
        serialize = "Lisp",
        serialize = "Windows",
        serialize = "informatics",
        serialize = "computer-sciences",
        serialize = "computer",
        serialize = "computational",
        serialize = "computing",
        serialize = "computer-hardware",
        serialize = "computer-software",
        serialize = "computer-theory",
        serialize = "computer-graphics",
        serialize = "computer-languages",
        serialize = "computing-theory",
        serialize = "information",
        serialize = "information-science",
        serialize = "information-technology",
        serialize = "information-theory",
        serialize = "software",
        serialize = "graphical-user-interface",
        serialize = "LISP",
        serialize = "databases",
        serialize = "demoscene",
        serialize = "software-compilation",
    )]
    Comp = 27,
    /// Construction / Bauwesen
    #[strum(
        to_string = "constr.",
        serialize = "CONSTR.",
        serialize = "constr",
        serialize = "CONSTR",
        serialize = "construction",
    )]
    Constr = 28,
    /// Cooking
    #[strum(
        to_string = "cook.",
        serialize = "COOK.",
        serialize = "COOK",
        serialize = "cook",
        serialize = "cooking",
        serialize = "baking",
        serialize = "cuisine",
        serialize = "Chinese-cuisine",
        serialize = "Indian-cookery",
        serialize = "Indian-Chinese-cuisine",
    )]
    Cook = 29,
    /// Cosmetics & Body Care / Kosmetik und Körperpflege
    #[strum(
        to_string = "cosmet.",
        serialize = "COSMET.",
        serialize = "COSMET",
        serialize = "cosmet",
        serialize = "cosmetics",
    )]
    Cosmet = 30,
    /// Currencies / Währungen
    #[strum(
        to_string = "curr.",
        serialize = "CURR.",
        serialize = "curr",
        serialize = "CURR"
    )]
    Curr = 31,
    /// Dance / Tanz
    #[strum(
        to_string = "dance",
        serialize = "DANCE.",
        serialize = "dance.",
        serialize = "DANCE"
    )]
    Dance = 32,
    /// Dental Medicine / Zahnmedizin
    #[strum(
        to_string = "dent.",
        serialize = "DENT.",
        serialize = "DENT",
        serialize = "dent",
        serialize = "dentistry",
    )]
    Dent = 33,
    /// Drugs / Drogen
    #[strum(
        to_string = "drugs",
        serialize = "DRUGS.",
        serialize = "DRUGS",
        serialize = "drugs."
    )]
    Drugs = 34,
    /// Ecology, Environment / Ökologie, Umwelt
    #[strum(
        to_string = "ecol.",
        serialize = "envir",
        serialize = "ENVIR.",
        serialize = "ecol",
        serialize = "ECOL",
        serialize = "ENVIR",
        serialize = "envir.",
        serialize = "ECOL.",
        serialize = "ecology",
    )]
    Ecol = 35,
    /// Economy / Wirtschaft, Ökonomie
    #[strum(
        to_string = "econ.",
        serialize = "econ",
        serialize = "ECON.",
        serialize = "ECON",
        serialize = "economic-liberalism",
        serialize = "economics",
        serialize = "microeconomics",
        serialize = "trading",
        serialize = "stock-ticker-symbol",
        serialize = "stock-market",
        serialize = "stock-exchange",
        serialize = "capitalism",
    )]
    Econ = 36,
    /// Education / Ausbildung
    #[strum(
        to_string = "educ.",
        serialize = "EDUC",
        serialize = "EDUC.",
        serialize = "educ",
        serialize = "education",
        serialize = "colleges",
        serialize = "higher-education",
    )]
    Educ = 37,
    /// Electrical Engin., Electronics / Elektrotechnik, Elektronik
    #[strum(
        to_string = "electr.",
        serialize = "ELECTR.",
        serialize = "ELECTR",
        serialize = "electr",
        serialize = "elect.",
        serialize = "electricity",
        serialize = "electromagnetism",
        serialize = "electrical",
        serialize = "robotics",
        serialize = "electronics",
        serialize = "electronics-manufacturing",
        serialize = "electrical-engineering",
        serialize = "lithography",
    )]
    Electr = 38,
    /// Engineering / Ingenieurwissenschaften
    #[strum(
        to_string = "engin.",
        serialize = "ENGIN.",
        serialize = "engin",
        serialize = "ENGIN",
        serialize = "engineering",
    )]
    Engin = 39,
    /// Entomology / Entomologie, Insektenkunde
    #[strum(
        to_string = "entom.",
        serialize = "entom",
        serialize = "ENTOM.",
        serialize = "ENTOM",
        serialize = "entomology",
    )]
    Entom = 40,
    /// Equestrianism, Horses / Reitsport, Pferde
    #[strum(
        to_string = "equest.",
        serialize = "EQUEST.",
        serialize = "equest",
        serialize = "EQUEST",
        serialize = "equestrianism",
        serialize = "horses",
        serialize = "horseracing",
        serialize = "horse-racing",
    )]
    Equest = 41,
    /// Esotericism / Esoterik
    #[strum(
        to_string = "esot.",
        serialize = "ESOT.",
        serialize = "esot",
        serialize = "ESOT",
        serialize = "tarot",
        serialize = "occultism",
    )]
    Esot = 42,
    /// Ethnology / Ethnologie
    #[strum(
        to_string = "ethn.",
        serialize = "ETHN.",
        serialize = "ETHN",
        serialize = "ethn",
        serialize = "ethnography",
        serialize = "ethnology",
    )]
    Ethn = 43,
    /// European Union / Europäische Union
    #[strum(
        to_string = "EU",
        serialize = "eu",
        serialize = "EU.",
        serialize = "eu."
    )]
    Eu = 44,
    /// Fiction: Names and Titles in Literature, Film, TV, Arts / Fiktion: Namen und Titel in Literatur, Film, TV, Kunst
    #[strum(
        to_string = "F",
        serialize = "F.",
        serialize = "f.",
        serialize = "f",
        serialize = "Harry-Potter",
    )]
    F = 45,
    /// Film / Film
    #[strum(
        to_string = "film",
        serialize = "FILM",
        serialize = "film.",
        serialize = "FILM."
    )]
    Film = 46,
    /// Finance / Finanzwesen
    #[strum(
        to_string = "fin.",
        serialize = "FIN.",
        serialize = "fin",
        serialize = "FIN",
        serialize = "finance",
        serialize = "financial",
        serialize = "finances"
    )]
    Fin = 47,
    /// Firefighting & Rescue / Feuerwehr & Rettungsdienst
    #[strum(
        to_string = "FireResc",
        serialize = "FIRERESC",
        serialize = "FireResc.",
        serialize = "FIRERESC.",
        serialize = "fireresc",
        serialize = "fireresc."
    )]
    FireResc = 48,
    /// Ichthyology, fish, fishing / Fischkunde, Fischen, Angelsport
    #[strum(
        to_string = "fish",
        serialize = "fish.",
        serialize = "FISH.",
        serialize = "FISH",
        serialize = "fisheries",
        serialize = "fishing",
    )]
    Fish = 49,
    /// Foodstuffs Industry / Lebensmittelindustrie
    #[strum(
        to_string = "FoodInd.",
        serialize = "FOODIND",
        serialize = "FOODIND.",
        serialize = "foodind.",
        serialize = "FoodInd",
        serialize = "foodind",
        serialize = "sugar-making",
    )]
    FoodInd = 50,
    /// Forestry / Forstwissenschaft, Forstwirtschaft
    #[strum(
        to_string = "for.",
        serialize = "FOR.",
        serialize = "for",
        serialize = "FOR",
        serialize = "forestry",
    )]
    For = 51,
    /// Furniture / Möbel
    #[strum(
        to_string = "furn.",
        serialize = "FURN",
        serialize = "furn",
        serialize = "FURN.",
        serialize = "furniture",
    )]
    Furn = 52,
    /// Games / Spiele
    #[strum(
        to_string = "games",
        serialize = "chess",
        serialize = "blackjack",
        serialize = "checkers",
        serialize = "video-games",
        serialize = "card-games",
        serialize = "GAMES.",
        serialize = "GAMES",
        serialize = "roguelikes",
        serialize = "role-playing-games",
        serialize = "games.",
        serialize = "computer-games",
    )]
    Games = 53,
    /// Gastronomy, Cooking / Gastronomie, Kochen
    #[strum(
        to_string = "gastr.",
        serialize = "gastr",
        serialize = "GASTR",
        serialize = "GASTR.",
    )]
    Gastr = 54,
    /// Geography / Geografie
    #[strum(
        to_string = "geogr.",
        serialize = "geogr",
        serialize = "GEOGR",
        serialize = "GEOGR.",
        serialize = "geography",
        serialize = "geographical-region",
        serialize = "topography",
    )]
    Geogr = 55,
    /// Geology / Geologie
    #[strum(
        to_string = "geol.",
        serialize = "GEOL.",
        serialize = "geol",
        serialize = "GEOL",
        serialize = "geology",
        serialize = "pedology",
    )]
    Geol = 56,
    /// Heraldry / Heraldik
    #[strum(
        to_string = "herald.",
        serialize = "HERALD.",
        serialize = "herald",
        serialize = "HERALD",
        serialize = "heraldry",
    )]
    Herald = 57,
    /// History / Historische Begriffe, Geschichte
    #[strum(
        to_string = "hist.",
        serialize = "HIST",
        serialize = "HIST.",
        serialize = "hist",
        serialize = "historical-demography",
        serialize = "history",
        serialize = "art-history",
    )]
    Hist = 58,
    /// Horticulture / Gartenbau
    #[strum(
        to_string = "hort.",
        serialize = "HORT.",
        serialize = "hort",
        serialize = "HORT",
        serialize = "horticulture",
    )]
    Hort = 59,
    /// Hunting / Jagd
    #[strum(
        to_string = "hunting",
        serialize = "HUNTING",
        serialize = "hunting.",
        serialize = "HUNTING.",
        serialize = "Jägersprache",
        serialize = "hunter's parlance",
        serialize = "hunters' parlance",
    )]
    Hunting = 60,
    /// Hydrology & Hydrogeology / Hydrologie & Hydrogeologie
    #[strum(
        to_string = "hydro.",
        serialize = "HYDRO",
        serialize = "hydro",
        serialize = "HYDRO.",
        serialize = "hydrology",
        serialize = "hydrography",
        serialize = "limnology",
    )]
    Hydro = 61,
    /// Idiom / Idiom, Redewendung
    #[strum(
        to_string = "idiom",
        serialize = "idiom.",
        serialize = "IDIOM",
        serialize = "IDIOM.",
        serialize = "Redewendung",
        serialize = "Sprw."
    )]
    Idiom = 62,
    /// Industry / Industrie
    #[strum(
        to_string = "ind.",
        serialize = "IND.",
        serialize = "ind",
        serialize = "IND"
    )]
    Ind = 63,
    /// Insurance / Versicherungswesen
    #[strum(
        to_string = "insur.",
        serialize = "INSUR",
        serialize = "insur",
        serialize = "INSUR.",
        serialize = "insurance",
    )]
    Insur = 64,
    /// Internet / Internet
    #[strum(
        to_string = "Internet",
        serialize = "internet.",
        serialize = "Internet.",
        serialize = "INTERNET",
        serialize = "internet",
        serialize = "INTERNET."
    )]
    Internet = 65,
    /// Jobs, Employment Market / Berufe, Arbeitsmarkt
    #[strum(
        to_string = "jobs",
        serialize = "JOBS.",
        serialize = "JOBS",
        serialize = "jobs.",
    )]
    Jobs = 66,
    /// Journalism / Journalismus
    #[strum(
        to_string = "journ.",
        serialize = "journ",
        serialize = "JOURN",
        serialize = "JOURN.",
        serialize = "journalism",
    )]
    Journ = 67,
    /// Law / Jura, Rechtswesen
    #[strum(
        to_string = "law",
        serialize = "intellectual-property",
        serialize = "law.",
        serialize = "LAW.",
        serialize = "LAW",
        serialize = "copyright",
        serialize = "court",
        serialize = "legal",
        serialize = "criminology",
        serialize = "jur.",
        serialize = "law-enforcement",
        serialize = "police",
        serialize = "patent-law",
    )]
    Law = 68,
    /// Library Science / Bibliothekswissenschaft
    #[strum(
        to_string = "libr.",
        serialize = "LIBR.",
        serialize = "LIBR",
        serialize = "libr",
        serialize = "bibliography",
    )]
    Libr = 69,
    /// Linguistics / Linguistik, Sprachwissenschaft
    #[strum(
        to_string = "ling.",
        serialize = "LING.",
        serialize = "ling",
        serialize = "LING",
        serialize = "grammar",
        serialize = "sociolinguistics",
        serialize = "linguistic",
        serialize = "lexicography",
        serialize = "linguistics",
        serialize = "linguistic-morphology",
        serialize = "psycholinguistics",
    )]
    Ling = 70,
    /// Literature / Literatur
    #[strum(
        to_string = "lit.",
        serialize = "LIT.",
        serialize = "LIT",
        serialize = "lit",
        serialize = "literature",
    )]
    Lit = 71,
    /// Machines
    #[strum(
        to_string = "mach.",
        serialize = "MACH.",
        serialize = "mach",
        serialize = "MACH",
        serialize = "machining",
    )]
    Mach = 72,
    /// Marketing, Advertising / Marketing, Werbung, Vertrieb und Handelswesen
    #[strum(
        to_string = "market.",
        serialize = "MARKET",
        serialize = "market",
        serialize = "MARKET.",
        serialize = "marketing",
    )]
    Market = 73,
    /// Materials Science / Materialwissenschaft, Werkstoffkunde
    #[strum(
        to_string = "material",
        serialize = "MATERIAL",
        serialize = "MATERIAL.",
        serialize = "material."
    )]
    Material = 74,
    /// Mathematics / Mathematik
    #[strum(
        to_string = "math.",
        serialize = "MATH.",
        serialize = "math",
        serialize = "MATH",
        serialize = "linear-algebra",
        serialize = "complex-analysis",
        serialize = "mathematics",
        serialize = "mathematical-analysis",
        serialize = "statistics",
        serialize = "trigonometry",
        serialize = "algebra",
        serialize = "algebraic-geometry",
        serialize = "algebraic-topology",
        serialize = "arithmetics",
        serialize = "arithmetic",
        serialize = "topology",
        serialize = "logic",
        serialize = "probability",
        serialize = "probability-theory",
    )]
    Math = 75,
    /// Medicine / Medizin
    #[strum(
        to_string = "med.",
        serialize = "MED.",
        serialize = "med",
        serialize = "MED",
        serialize = "surgery",
        serialize = "pathology",
        serialize = "pulmonology",
        serialize = "anesthesiology",
        serialize = "gastroenterology",
        serialize = "medicine",
        serialize = "medical-terminology",
        serialize = "histology",
        serialize = "histopathology",
        serialize = "physiology",
        serialize = "teratology",
        serialize = "traumatology",
        serialize = "gynaecology",
        serialize = "illness",
        serialize = "immunology",
        serialize = "health",
        serialize = "radiography",
        serialize = "radiology",
        serialize = "healthcare",
        serialize = "cytology",
        serialize = "dermatology",
        serialize = "epidemiology",
    )]
    Med = 76,
    /// Medical Engineering & Imaging / Medizintechnik
    #[strum(
        to_string = "MedTech.",
        serialize = "MEDTECH.",
        serialize = "MEDTECH",
        serialize = "medtech",
        serialize = "medtech.",
        serialize = "MedTech"
    )]
    MedTech = 77,
    /// Meteorology / Meteorologie
    #[strum(
        to_string = "meteo.",
        serialize = "METEO.",
        serialize = "meteo",
        serialize = "METEO",
        serialize = "meteorology",
    )]
    Meteo = 78,
    /// Military / Militärwesen
    #[strum(
        to_string = "mil.",
        serialize = "MIL",
        serialize = "MIL.",
        serialize = "mil",
        serialize = "Soldatensprache",
        serialize = "milit.",
        serialize = "fortifications",
        serialize = "military",
        serialize = "army",
        serialize = "war",
    )]
    Mil = 79,
    /// Mineralogy / Mineralogie
    #[strum(
        to_string = "mineral.",
        serialize = "mineral",
        serialize = "MINERAL.",
        serialize = "MINERAL",
        serialize = "mineralology",
        serialize = "mineralogy",
    )]
    Mineral = 80,
    /// Mining & Drilling / Bergbau & Bohrtechnik
    #[strum(
        to_string = "mining",
        serialize = "min.",
        serialize = "MIN.",
        serialize = "MINING",
        serialize = "MIN",
        serialize = "mining.",
        serialize = "min",
        serialize = "MINING."
    )]
    Mining = 81,
    /// Music / Musik
    #[strum(
        to_string = "mus.",
        serialize = "MUS.",
        serialize = "MUS",
        serialize = "mus",
        serialize = "music",
        serialize = "musicology",
        serialize = "hip-hop",
        serialize = "guitar",
        serialize = "lutherie",
    )]
    Mus = 82,
    /// Mycology / Mykologie, Pilze
    #[strum(
        to_string = "mycol.",
        serialize = "MYCOL.",
        serialize = "MYCOL",
        serialize = "mycol",
        serialize = "myc.",
        serialize = "mycology",
        serialize = "lichenology",
    )]
    Mycol = 83,
    /// Mythology / Mythologie
    #[strum(
        to_string = "myth.",
        serialize = "MYTH.",
        serialize = "myth",
        serialize = "MYTH",
        serialize = "mythology",
    )]
    Myth = 84,
    /// Names of Persons / Namenkunde (nur Personennamen)
    #[strum(
        to_string = "name",
        serialize = "NAME",
        serialize = "name.",
        serialize = "NAME."
    )]
    Name = 85,
    /// Nautical Science / Nautik, Schifffahrtskunde
    #[strum(
        to_string = "naut.",
        serialize = "NAUT",
        serialize = "naut",
        serialize = "NAUT.",
        serialize = "nautical",
    )]
    Naut = 86,
    /// Neologisms / Neologismen (Wortneubildungen)
    #[strum(
        to_string = "neol.",
        serialize = "neol",
        serialize = "NEOL.",
        serialize = "NEOL"
    )]
    Neol = 87,
    /// Nuclear Engineering / Nukleartechnik
    #[strum(
        to_string = "nucl.",
        serialize = "NUCL",
        serialize = "nucl",
        serialize = "NUCL."
    )]
    Nucl = 88,
    /// Oenology / Önologie, Lehre vom Wein
    #[strum(
        to_string = "oenol.",
        serialize = "OENOL.",
        serialize = "oenol",
        serialize = "OENOL",
        serialize = "oenology",
    )]
    Oenol = 89,
    /// Optics / Optik
    #[strum(
        to_string = "optics",
        serialize = "OPTICS.",
        serialize = "optics.",
        serialize = "OPTICS",
        serialize = "optical"
    )]
    Optics = 90,
    /// Ornithology / Ornithologie, Vogelkunde
    #[strum(
        to_string = "orn.",
        serialize = "orn",
        serialize = "ORN.",
        serialize = "ORN",
        serialize = "ornith.",
        serialize = "ORNITH.",
        serialize = "ornith",
        serialize = "ORNITH",
        serialize = "Ornithology",
        serialize = "ornithology",
    )]
    Orn = 91,
    /// Pharmacy / Pharmazie
    #[strum(
        to_string = "pharm.",
        serialize = "PHARM.",
        serialize = "pharm",
        serialize = "PHARM",
        serialize = "pharmaceuticals",
        serialize = "pharmacology",
        serialize = "toxicology",
    )]
    Pharm = 92,
    /// Philately / Philatelie, Briefmarkenkunde
    #[strum(
        to_string = "philat.",
        serialize = "philat",
        serialize = "PHILAT",
        serialize = "PHILAT."
    )]
    Philat = 93,
    /// Philosophy / Philosophie
    #[strum(
        to_string = "philos.",
        serialize = "phil.",
        serialize = "PHILOS",
        serialize = "phil",
        serialize = "PHIL",
        serialize = "PHIL.",
        serialize = "philos",
        serialize = "PHILOS.",
        serialize = "philosophy",
        serialize = "socialism",
        serialize = "communism",
    )]
    Philos = 94,
    /// Phonetics / Phonetik
    #[strum(
        to_string = "phonet.",
        serialize = "PHONET.",
        serialize = "PHONET",
        serialize = "phonet",
        serialize = "phonetics",
    )]
    Phonet = 95,
    /// Photography / Fotografie
    #[strum(
        to_string = "photo.",
        serialize = "PHOTO",
        serialize = "photo",
        serialize = "PHOTO.",
        serialize = "photography",
    )]
    Photo = 96,
    /// Physics / Physik
    #[strum(
        to_string = "phys.",
        serialize = "PHYS.",
        serialize = "phys",
        serialize = "PHYS",
        serialize = "hydrodynamics",
        serialize = "physics",
        serialize = "physical-sciences",
        serialize = "thermodynamics",
        serialize = "crystallography",
    )]
    Phys = 97,
    /// Politics / Politik
    #[strum(
        to_string = "pol.",
        serialize = "POL",
        serialize = "POL.",
        serialize = "pol",
        serialize = "sociopolitics",
        serialize = "political-science",
        serialize = "politics",
        serialize = "state",
        serialize = "diplomacy",
        serialize = "monarchy",
        serialize = "anarchism",
        serialize = "government",
    )]
    Pol = 98,
    /// Print, Typography, Layout / Druck, Typografie, Layout
    #[strum(
        to_string = "print",
        serialize = "print.",
        serialize = "PRINT.",
        serialize = "PRINT",
        serialize = "printing-technology",
        serialize = "printing",
        serialize = "typography",
        serialize = "stenography",
        serialize = "letterpress-typography",
    )]
    Print = 99,
    /// Proverb / Sprichwort
    #[strum(
        to_string = "proverb",
        serialize = "PROVERB",
        serialize = "PROVERB.",
        serialize = "proverb.",
        serialize = "prov.",
    )]
    Proverb = 100,
    /// Psychology / Psychologie
    #[strum(
        to_string = "psych.",
        serialize = "PSYCH.",
        serialize = "psych",
        serialize = "PSYCH",
        serialize = "psychiatry",
        serialize = "psychoanalysis",
        serialize = "psychology",
        serialize = "psychopathology",
    )]
    Psych = 101,
    /// Publishing / Verlagswesen
    #[strum(
        to_string = "publ.",
        serialize = "publ",
        serialize = "PUBL",
        serialize = "PUBL.",
        serialize = "publishing",
    )]
    Publ = 102,
    /// Quality Management / Qualitätsmanagement
    #[strum(
        to_string = "QM",
        serialize = "qm.",
        serialize = "QM.",
        serialize = "qm",
    )]
    Qm = 103,
    /// Quotation / Zitat
    #[strum(
        to_string = "quote",
        serialize = "QUOTE",
        serialize = "QUOTE.",
        serialize = "quote.",
    )]
    Quote = 104,
    /// Radio and Television / Radio und Fernsehen
    #[strum(
        to_string = "RadioTV",
        serialize = "RADIOTV",
        serialize = "tv",
        serialize = "TV.",
        serialize = "RadioTV.",
        serialize = "RADIOTV.",
        serialize = "tv.",
        serialize = "radiotv",
        serialize = "TV",
        serialize = "radiotv.",
        serialize = "television",
        serialize = "radio",
        serialize = "radio-communications",
        serialize = "radio-technics",
        serialize = "radio-technology",
    )]
    RadioTv = 105,
    /// Rail / Eisenbahn
    #[strum(
        to_string = "rail",
        serialize = "RAIL.",
        serialize = "RAIL",
        serialize = "rail.",
        serialize = "rail-transport",
        serialize = "railways",
        serialize = "trains",
    )]
    Rail = 106,
    /// Real Estate / Immobilien
    #[strum(
        to_string = "RealEst.",
        serialize = "REALEST.",
        serialize = "RealEst",
        serialize = "realest.",
        serialize = "realest",
        serialize = "REALEST",
    )]
    RealEst = 107,
    /// Religion / Religion
    #[strum(
        to_string = "relig.",
        serialize = "relig",
        serialize = "region",
        serialize = "RELIG",
        serialize = "RELIG.",
        serialize = "religion",
        serialize = "Catholic",
        serialize = "Christian",
        serialize = "Hinduism",
        serialize = "Hebrew",
        serialize = "Church-of-England",
        serialize = "paganism",
        serialize = "mysticism",
        serialize = "hinduism",
        serialize = "Roman-Catholicism",
        serialize = "spiritualism",
        serialize = "theology",
        serialize = "Islam",
        serialize = "Scientology",
        serialize = "Shinto",
        serialize = "Buddhist",
        serialize = "Western-Christianity",
        serialize = "Christianity",
        serialize = "Buddhism",
        serialize = "Abrahamic-religions",
        serialize = "Eastern-Christianity",
        serialize = "creationism",
        serialize = "ecclesiastical",
        serialize = "Gnosticism",
        serialize = "Protestantism",
        serialize = "Sufism",
        serialize = "Wicca",
        serialize = "Zoroastrianism",
        serialize = "Catholicism",
        serialize = "ethics",
        serialize = "Mormonism",
    )]
    Relig = 108,
    /// Rhetoric / Rhetorik
    #[strum(
        to_string = "rhet.",
        serialize = "rhet",
        serialize = "RHET.",
        serialize = "RHET",
    )]
    Rhet = 109,
    /// School/Schule
    #[strum(
        to_string = "school",
        serialize = "SCHOOL.",
        serialize = "SCHOOL",
        serialize = "school.",
    )]
    School = 110,
    /// Sociology / Soziologie
    #[strum(
        to_string = "sociol.",
        serialize = "SOC",
        serialize = "SOC.",
        serialize = "SOCIOL.",
        serialize = "sociol",
        serialize = "soc",
        serialize = "SOCIOL",
        serialize = "soc.",
        serialize = "sociology",
        serialize = "social-sciences",
        serialize = "social-science",
    )]
    Sociol = 111,
    /// Specialized Term / Fachsprachlicher Ausdruck
    #[strum(
        to_string = "spec.",
        serialize = "spec",
        serialize = "SPEC",
        serialize = "SPEC.",
    )]
    Spec = 112,
    /// Sports / Sport
    #[strum(
        to_string = "sports",
        serialize = "sport",
        serialize = "handball",
        serialize = "underwater-diving",
        serialize = "bullfighting",
        serialize = "cheerleading",
        serialize = "dancing",
        serialize = "aerial-freestyle",
        serialize = "pesäpallo",
        serialize = "professional-wrestling",
        serialize = "juggling",
        serialize = "ballooning",
        serialize = "paintball",
        serialize = "archery",
        serialize = "sailing",
        serialize = "darts",
        serialize = "American-football",
        serialize = "(sport)",
        serialize = "SPORT.",
        serialize = "SPORTS.",
        serialize = "SPORTS",
        serialize = "SPORT",
        serialize = "snooker",
        serialize = "softball",
        serialize = "sport.",
        serialize = "sports.",
        serialize = "soccer.",
        serialize = "soccer",
        serialize = "baseball",
        serialize = "basketball",
        serialize = "judo",
        serialize = "archer",
        serialize = "arena",
        serialize = "arrow",
        serialize = "athlete",
        serialize = "axel",
        serialize = "badminton",
        serialize = "ball",
        serialize = "base",
        serialize = "bat",
        serialize = "batter",
        serialize = "bicycle",
        serialize = "bocce",
        serialize = "bow",
        serialize = "box",
        serialize = "canoe",
        serialize = "catch",
        serialize = "cleats",
        serialize = "club",
        serialize = "coach",
        serialize = "compete",
        serialize = "crew",
        serialize = "cricket",
        serialize = "cycle",
        serialize = "cyclist",
        serialize = "dart",
        serialize = "defense",
        serialize = "diamond",
        serialize = "dive",
        serialize = "diver",
        serialize = "exercise",
        serialize = "fencing",
        serialize = "field",
        serialize = "fitness",
        serialize = "frisbee",
        serialize = "game",
        serialize = "gear",
        serialize = "goal",
        serialize = "goalie",
        serialize = "golf",
        serialize = "golfer",
        serialize = "guard",
        serialize = "gym",
        serialize = "gymnast",
        serialize = "helmet",
        serialize = "hockey",
        serialize = "home",
        serialize = "hoop",
        serialize = "hoops",
        serialize = "ice",
        serialize = "infield",
        serialize = "inning",
        serialize = "javelin",
        serialize = "jog",
        serialize = "jump",
        serialize = "jumper",
        serialize = "karate",
        serialize = "kayak",
        serialize = "kite",
        serialize = "lacrosse",
        serialize = "league",
        serialize = "lose",
        serialize = "loser",
        serialize = "luge",
        serialize = "major",
        serialize = "mallet",
        serialize = "mat",
        serialize = "medal",
        serialize = "mitt",
        serialize = "move",
        serialize = "net",
        serialize = "offense",
        serialize = "olympics",
        serialize = "out",
        serialize = "paddle",
        serialize = "pitch",
        serialize = "play",
        serialize = "player",
        serialize = "pole",
        serialize = "polo",
        serialize = "pool",
        serialize = "puck",
        serialize = "quarter",
        serialize = "quiver",
        serialize = "race",
        serialize = "racer",
        serialize = "referee",
        serialize = "relay",
        serialize = "ride",
        serialize = "rink",
        serialize = "row",
        serialize = "rower",
        serialize = "sail",
        serialize = "score",
        serialize = "scuba",
        serialize = "skate",
        serialize = "ski",
        serialize = "skier",
        serialize = "slalom",
        serialize = "sled",
        serialize = "sledder",
        serialize = "snowboard",
        serialize = "netball",
        serialize = "boxing",
        serialize = "ball-games",
        serialize = "gymnastics",
        serialize = "skateboarding",
        serialize = "snowboarding",
        serialize = "rowing",
        serialize = "weightlifting",
        serialize = "skiing",
        serialize = "swimming",
        serialize = "squash",
        serialize = "stadium",
        serialize = "stick",
        serialize = "sumo",
        serialize = "surfer",
        serialize = "surfing",
        serialize = "volleyball",
        serialize = "ice-hockey",
        serialize = "bowling",
        serialize = "swim",
        serialize = "swimmer",
        serialize = "tag",
        serialize = "target",
        serialize = "team",
        serialize = "tee",
        serialize = "tennis",
        serialize = "throw",
        serialize = "tie",
        serialize = "triathlon",
        serialize = "umpire",
        serialize = "vault",
        serialize = "volley",
        serialize = "walk",
        serialize = "weight",
        serialize = "win",
        serialize = "winner",
        serialize = "winning",
        serialize = "wrestler",
        serialize = "curling",
        serialize = "cycling",
    )]
    Sports = 113,
    /// Statistics / Statistik
    #[strum(
        to_string = "stat.",
        serialize = "STAT",
        serialize = "STATIST.",
        serialize = "STAT.",
        serialize = "stat",
        serialize = "STATIST",
        serialize = "statist",
        serialize = "statist.",
    )]
    Stat = 114,
    /// Stock Exchange / Börsenwesen
    #[strum(
        to_string = "stocks",
        serialize = "STOCKS",
        serialize = "stocks.",
        serialize = "STOCKS.",
    )]
    Stocks = 115,
    /// Studium
    #[strum(
        to_string = "stud.",
        serialize = "STUD",
        serialize = "stud",
        serialize = "STUD."
    )]
    Stud = 116,
    /// Taxonomic terms for animals, plants and fungi (incl. varieties and breeds) / Taxonomische Bezeichnungen für Tiere, Pflanzen und Pilze (inkl. Zuchtformen und Rassen)
    #[strum(
        to_string = "T",
        serialize = "t",
        serialize = "t.",
        serialize = "T.",
        serialize = "taxonomy",
    )]
    T = 117,
    /// Technology / Technik
    #[strum(
        to_string = "tech.",
        serialize = "TECH",
        serialize = "tech",
        serialize = "TECH.",
        serialize = "technology",
        serialize = "technical",
        serialize = "in-technical-contexts",
    )]
    Tech = 118,
    /// Telecommunications / Telekommunikation
    #[strum(
        to_string = "telecom.",
        serialize = "TELCO",
        serialize = "TELECOM.",
        serialize = "TELECOM",
        serialize = "TELCO.",
        serialize = "telco",
        serialize = "telecom",
        serialize = "telco.",
        serialize = "telecommunications",
        serialize = "telegraphy",
        serialize = "telephony",
        serialize = "telephone",
        serialize = "mobile-telephony",
    )]
    Telecom = 119,
    /// Textiles, Textile Industry / Textilien, Textilindustrie
    #[strum(
        to_string = "textil.",
        serialize = "TEXTIL",
        serialize = "textil",
        serialize = "TEXTIL.",
        serialize = "textiles",
    )]
    Textil = 120,
    /// Theatre / Theater
    #[strum(
        to_string = "theatre",
        serialize = "THEATRE.",
        serialize = "theatre.",
        serialize = "THEATRE",
    )]
    Theatre = 121,
    /// Tools / Werkzeuge
    #[strum(
        to_string = "tools",
        serialize = "TOOLS.",
        serialize = "tools.",
        serialize = "TOOLS",
    )]
    Tools = 122,
    /// Toys / Spielzeug
    #[strum(
        to_string = "toys",
        serialize = "TOYS",
        serialize = "toys.",
        serialize = "TOYS.",
    )]
    Toys = 123,
    /// Travellers vocabulary / Reise-Wortschatz
    #[strum(
        to_string = "TrVocab.",
        serialize = "TrVocab",
        serialize = "trvocab.",
        serialize = "trvocab",
        serialize = "TRVOCAB",
        serialize = "TRVOCAB.",
    )]
    TrVocab = 124,
    /// Traffic / Verkehrswesen
    #[strum(
        to_string = "traffic",
        serialize = "TRAFFIC",
        serialize = "TRAFFIC.",
        serialize = "traffic.",
    )]
    Traffic = 125,
    /// Transportation (Land Transport) / Transportwesen (Landtransport)
    #[strum(
        to_string = "transp.",
        serialize = "TRANSP.",
        serialize = "TRANSP",
        serialize = "transp",
        serialize = "transport",
    )]
    Transp = 126,
    /// Travel Industry / Touristik
    #[strum(
        to_string = "travel",
        serialize = "travel.",
        serialize = "TRAVEL",
        serialize = "TRAVEL.",
        serialize = "travel-industry",
    )]
    Travel = 127,
    /// Units, Measures, Weights / Einheiten, Maße, Gewichte
    #[strum(
        to_string = "unit",
        serialize = "UNIT",
        serialize = "UNIT.",
        serialize = "unit.",
        serialize = "units-of-measure",
        serialize = "time",
        serialize = "temperature",
    )]
    Unit = 128,
    /// Urban Planning / Urbanistik, Stadtplanung
    #[strum(
        to_string = "urban",
        serialize = "URBAN",
        serialize = "URBAN.",
        serialize = "urban.",
        serialize = "urbanism",
    )]
    Urban = 129,
    /// UNESCO World Heritage / UNESCO-Welterbe
    #[strum(
        to_string = "UWH",
        serialize = "uwh.",
        serialize = "uwh",
        serialize = "UWH."
    )]
    Uwh = 130,
    /// Veterinary Medicine / Veterinärmedizin
    #[strum(
        to_string = "VetMed.",
        serialize = "vetmed.",
        serialize = "VetMed",
        serialize = "vetmed",
        serialize = "VETMED.",
        serialize = "VETMED",
        serialize = "zootomy",
        serialize = "veterinary",
        serialize = "veterinary-pathology",
    )]
    VetMed = 131,
    /// Watches, Clocks / Uhren
    #[strum(
        to_string = "watches",
        serialize = "WATCHES.",
        serialize = "WATCHES",
        serialize = "watches.",
    )]
    Watches = 132,
    /// Weapons / Waffen
    #[strum(
        to_string = "weapons",
        serialize = "weapons.",
        serialize = "WEAPONS",
        serialize = "WEAPONS.",
        serialize = "firearms",
        serialize = "weapon",
        serialize = "weaponry",
    )]
    Weapons = 133,
    /// Zoology, Animals / Zoologie, Tierkunde
    #[strum(
        to_string = "zool.",
        serialize = "ZOOL.",
        serialize = "ZOOL",
        serialize = "zool",
        serialize = "zoology",
        serialize = "conchology",
        serialize = "mammalogy",
        serialize = "mammology",
    )]
    Zool = 134,
    /// Kindersprache
    #[strum(
        to_string = "children's speech",
        serialize = "Kindersprache",
    )]
    Child = 135,
    /// Kindersprache
    #[strum(
        to_string = "youth speech",
        serialize = "Jugendsprache",
    )]
    Youth = 136,
    /// Wissenschaft
    #[strum(
        to_string = "sci.",
        serialize = "sciences",
        serialize = "science",
    )]
    Science = 137,
    /// Wissenschaft
    #[strum(
        to_string = "poet.",
    )]
    Poetry = 138,
    /// Wissenschaft
    #[strum(
        to_string = "currency",
    )]
    Currency = 139,
    /// Philatelie
    #[strum(
        to_string = "philately",
    )]
    Phila = 140,
    /// Kommunikation
    #[strum(
        to_string = "communication",
        serialize = "communications",
    )]
    Commun = 141,
    #[strum(
        to_string = "media",
    )]
    Media = 142,
    #[strum(
        to_string = "tourism",
    )]
    Tour = 143,
    #[strum(
        to_string = "alchemy",
    )]
    Alchemy = 144,
    #[strum(
        to_string = "anime",
        serialize = "manga",
    )]
    Anime = 145,
    #[strum(
        to_string = "beverages",
    )]
    Bever = 146,
    #[strum(
        to_string = "sex",
        serialize = "Sex",
        serialize = "sexology",
        serialize = "sexuality",
        serialize = "prostitution",
        serialize = "pornography",
        serialize = "BDSM",
    )]
    Sex = 147,
    #[strum(
        to_string = "paleontology",
        serialize = "palentology",
        serialize = "paleoanthropology",
        serialize = "palaeography",
        serialize = "paleography",
        serialize = "palynology",
    )]
    Palaeo = 148,
    #[strum(
        to_string = "metallurgy",
        serialize = "metalworking",
        serialize = "ironworking",
        serialize = "smithwork",
    )]
    Metal = 149,
    #[strum(
        to_string = "masonry",
    )]
    Masonry = 150,
    #[strum(
        to_string =  "color",
        serialize = "colorimetry",
        serialize = "colour",
    )]
    Colour = 151,
    #[strum(
        to_string =  "mechanics",
        serialize = "mechanical-engineering",
        serialize = "mechanical",
    )]
    Mechanics = 152,
    #[strum(
        to_string =  "money",
    )]
    Money = 153,
    #[strum(
        to_string =  "natural-sciences",
    )]
    NatSci = 154,
    #[strum(
        to_string =  "pseudoscience",
        serialize = "homeopathy",
        serialize = "cryptozoology",
    )]
    PseudoSci = 155,
    #[strum(
        to_string =  "human-sciences",
    )]
    Humanities = 156,
}

impl_try_from_as_unpack! {
    Domain => Domain
}

// #[cfg_attr(feature="gen_python_api", pyo3_stub_gen::derive::gen_stub_pymethods)]
#[pymethods]
impl Domain {
    fn __str__(&self) -> &'static str {
        self.into()
    }

    fn __repr__(&self) -> &'static str {
        self.into()
    }
}

impl DomainModelIndex for Domain {
    #[inline(always)]
    fn as_index(self) -> usize {
        (self as u16) as usize
    }

    fn from_index(index: usize) -> Result<Self, NotAIndexFor> {
        Domain::try_from(index as u16).map_err(|_| NotAIndexFor(stringify!(Domain), index))
    }
}

impl Fits64 for Domain {
    #[inline(always)]
    unsafe fn from_u64(x: u64) -> Self {
        Domain::try_from(x as u16).unwrap()
    }
    #[inline(always)]
    fn to_u64(self) -> u64 {
        (self as u16) as u64
    }
}