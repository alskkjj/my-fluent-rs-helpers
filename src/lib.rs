use fluent::{FluentArgs, FluentValue, FluentBundle, FluentResource};
use std::{
    io,
    env,
    fs,
    sync::{Mutex, Arc, OnceLock},
    io::Read,
    path::PathBuf,
};

use unic_langid::LanguageIdentifier;



/// fluent functions
#[derive(Debug, )]
pub enum LanguageChoiceError {
    IoError(io::Error),
    NoLanguageFilesAt(String),
    LanguageNegotiatedFailed(String, Vec<String>, ),
}

impl From<io::Error> for LanguageChoiceError {
    fn from(e: io::Error) -> Self {
        Self::IoError(e)
    }
}

fn language_matches_score(l1: &LanguageIdentifier, l2: &LanguageIdentifier) -> u8 {
    let mut base = 0u8;
    base |= if l1.matches(l2, false, false) { 0b1000 } else { 0 };
    base |= if l1.matches(l2, false, true) { 0b0100 } else { 0 };
    base |= if l1.matches(l2, true, false) { 0b0010 } else { 0 };
    base |= if l1.matches(l2, true, true) { 0b0001 } else { 0 };
    base
}

#[derive(Debug)]
struct LanguageDeductionHelperS {
    pub lid: LanguageIdentifier,
    pub lang_name: String,
    pub dir_path: PathBuf,
    pub score: u8,
}



pub struct LanguageSystem {
    pub bundle: fluent::FluentBundle<FluentResource>,
    pub current_lang: LanguageIdentifier,
    pub current_lang_dir_path: PathBuf,
    pub donnot_panic: bool,
}

unsafe impl Sync for LanguageSystem {}
unsafe impl Send for LanguageSystem {}

impl LanguageSystem {
    fn resolve_desired_lang(lang_name: Option<String>, lang_dir: &PathBuf, donnot_panic: bool) 
        -> Result<Vec<LanguageDeductionHelperS>, LanguageChoiceError> {
            if !lang_dir.exists() || !lang_dir.is_dir() {
                if donnot_panic { return Ok(Vec::new()); }
                return Err(LanguageChoiceError::NoLanguageFilesAt(
                        lang_dir.canonicalize()
                        .unwrap()
                        .to_string_lossy()
                        .into_owned()
                ));
            }

            let (desired_lang_identifier, desired_dirname) = match &lang_name {
                Some(lang) => {
                    (lang.parse::<LanguageIdentifier>()
                     .expect(&format!("MFH: Parse {lang} as language identifier failed.")),
                     lang.clone())
                },
                None => {
                    let n = sys_locale::get_locale()
                        .expect(&format!("MFH Get system locale failed."));
                    let li = n.clone()
                        .parse::<LanguageIdentifier>()
                        .expect("MFH: System's default locale parses failed.");
                    (li, n)
                }
            };

            let available_langs = {
                let mut available_langs = Vec::new();
                let read_dir = fs::read_dir(lang_dir)
                    .expect(&format!("MFH: Read dir {:?} failed.", lang_dir));
                for dir in read_dir {
                    let dir_ent = dir.expect(&format!("MFH: Read a dir entry in {:?} failed.", lang_dir));
                    let dir_path = dir_ent.path();

                    let dirname = {
                        let os_name = dir_ent.file_name();
                        os_name.to_str()
                            .expect(&format!("MFH: OsString {:?} converts to String failed.", &os_name)).to_owned()
                    };
                    match &dirname.parse::<LanguageIdentifier>() {
                        Ok(id) => {
                            let tmp = LanguageDeductionHelperS {
                                lid: id.clone(),
                                lang_name: dirname,
                                dir_path,
                                score: language_matches_score(&id, &desired_lang_identifier)
                            };
                            available_langs.push(tmp);
                        },
                        Err(_e) => {
                        }
                    }
                }
                available_langs.sort_by_cached_key(|a| { a.lang_name.clone() });
                available_langs.sort_by(|a, b| { b.score.cmp(&a.score) });
                available_langs
            };
            if !available_langs.is_empty() {
                Ok(available_langs)
            } else {
                Err(LanguageChoiceError::LanguageNegotiatedFailed(desired_dirname, available_langs.into_iter()
                        .map(|a| {
                            a.lang_name
                        })
                        .collect()
                ))
            }
        }
}

static LANG: OnceLock<Mutex<Arc<LanguageSystem>>> = OnceLock::new();

impl LanguageSystem {

    pub fn new(desired_lang: Option<String>, lang_dir: Option<String>, donnot_panic: bool) -> Self {
        let default_lang_dir_str = "i18n/fluent".to_owned();
        let lang_dir = lang_dir.or(Some(default_lang_dir_str)).unwrap();
        let lang_dir = {
            let mut tmp = env::current_dir()
                .expect("MFH: Get current dir failed.");
            tmp.extend(lang_dir.split("/"));
            tmp
        };

        let ordered_langs = Self::resolve_desired_lang(desired_lang.clone(), &lang_dir, donnot_panic)
            .expect(&format!("MFH: Fetch languages {:?} failed.", desired_lang));
        let v = ordered_langs
            .iter()
            .map(|a| { a.lid.clone() })
            .collect();
        let mut bundle = FluentBundle::new(v);
            
 //       let desired_lang_helper_s = .unwrap();
        match &ordered_langs.first() {
            Some(desired_lang_helper_s) => { // add ftl files under desired directory to bundle.
                let read_dir = fs::read_dir(&desired_lang_helper_s.dir_path)
                    .expect(&format!("MFH: Read language dir {:?} failed", &desired_lang_helper_s.dir_path));

                for entry in read_dir {
                    if let Ok(dir_entry) = entry {
                        let path = dir_entry.path();
                        if path.is_file() && path.extension().is_some()
                            && path.extension().unwrap() == "ftl" {
                                {
                                    let mut f = fs::File::open(path)
                                        .expect("MFH: Failed to open one of ftl files.");
                                    let mut s = String::new();
                                    f.read_to_string(&mut s).expect("read ftl file to string failed.");
                                    let r = FluentResource::try_new(s)
                                        .expect("MFH: Could not parse an FTL string.");
                                    bundle.add_resource(r)
                                        .expect("MFH: Failed to add FTL resources to the bundle.");
                                    }
                        }
                    }
                }

                Self {
                    bundle,
                    current_lang: desired_lang_helper_s.lid.clone(),
                    current_lang_dir_path: desired_lang_helper_s.dir_path.clone(),
                    donnot_panic,
                }
            },
            None => {
                if donnot_panic {
                    Self {
                        bundle,
                        current_lang: "en".parse::<LanguageIdentifier>().unwrap(),
                        current_lang_dir_path: PathBuf::new(),
                        donnot_panic,
                    }
                } else {
                    panic!("Ordered lang failed.")
                }
            }
        }
    }
}

const NO_SUCH_MESSAGE_ERROR: &str = "NoSuchMessage";
const NO_VALUE_FOUND_ERROR: &str = "NoValueFound";

fn make_error_replacement(err: &str, msg_to_show: &str) -> String {
    format!("<{} - {}>", err, msg_to_show)
}

pub fn build_language_0<'a>(msg_key: &str) -> String {
    match LANG.get()
        .expect("MFH: Uninitialized language bundle.").lock() {
        Ok(bs) => {
            
            let msg = match bs.bundle
                .get_message(msg_key) {
                    Some(msg) => msg,
                    None => {
                        if bs.donnot_panic {
                            return make_error_replacement(NO_SUCH_MESSAGE_ERROR, msg_key);
                        } else {
                            panic!("MFH: Failed to find message {}", msg_key);
                        }
                    }
                };
            let mut errors = vec![];
            let pattern = match msg.value() {
                Some(pt) => pt,
                None => {
                    if bs.donnot_panic {
                        return make_error_replacement(NO_VALUE_FOUND_ERROR, msg_key);
                    } else {
                        panic!("MFH: Message has no value.");
                    }
                }
            };
            let v = bs.bundle.format_pattern(pattern, None, &mut errors);
            v.to_string()
        },
        Err(e) => {
            panic!("MFH: Language bundle mutext poisoned. {e}");
        }
    }
}

pub fn build_language_1<'a, T>(msg_key: &str, arg_name: &str, v: T) -> String
    where T: Into<FluentValue<'a>> {
    build_language(msg_key, 
        vec![(arg_name, v.into())])
}

pub fn build_language_2<'a, T, R>(msg_key: &str, arg1_name: &str, v1: T, arg2_name: &str, v2: R) -> String
    where T: Into<FluentValue<'a>>,
          R: Into<FluentValue<'a>>,
{
    build_language(msg_key, 
        vec![(arg1_name, v1.into()),
        (arg2_name, v2.into()),
        ])
}

pub fn build_language_3<'a, T, R>(msg_key: &str, arg1_name: &str, v1: T, 
    arg2_name: &str, v2: R, 
    arg3_name: &str, v3: R) -> String
    where T: Into<FluentValue<'a>>,
          R: Into<FluentValue<'a>>,
{
    build_language(msg_key, 
        vec![(arg1_name, v1.into()),
        (arg2_name, v2.into()),
        (arg3_name, v3.into()),
        ])
}


pub fn build_language_fns<'a, F>(msg_key: &str, args_pairs_builders: Vec<(&str, F)>) -> String 
where F: FnOnce() -> FluentValue<'a>{
    let args_pairs: Vec<_> = args_pairs_builders.into_iter()
        .map(
            |a| {
            (a.0,
             a.1())
            }
            )
        .collect();
    build_language(msg_key, args_pairs)
}

fn concat_parameters_key(msg_key: &str, args_pairs: &[(&str, FluentValue)]) -> String {
    format!("[{}]",
        args_pairs.iter() 
        .map(|a| {
            let k = a.0;
            format!("{}", k)
        })
        .fold("<".to_owned() + &String::from(msg_key) + ">", |c, v| { c + "," + &v } )
    )
}

pub fn build_language<'a>(msg_key: &str, args_pairs: Vec<(&str, FluentValue)>) -> String {
    if let Ok(bs) = LANG.get().expect("MFH: Uninitialized language bundle.").lock() {
        let msg = match bs
            .bundle
            .get_message(msg_key) {
                Some(bs) => {bs}
                None => {
                    if bs.donnot_panic {
                        let msg_to_show = concat_parameters_key(msg_key, &args_pairs);
                        return make_error_replacement(NO_SUCH_MESSAGE_ERROR, &msg_to_show);
                    } else {
                        panic!("MFH: Failed to find message {}", msg_key);
                    }
                }
            };

        let pattern = match msg.value() {
            Some(pt) => pt,
            None => {
                if bs.donnot_panic {
                    let msg_to_show = concat_parameters_key(msg_key, &args_pairs);
                    return make_error_replacement(NO_VALUE_FOUND_ERROR, &msg_to_show);
                } else {
                    panic!("MFH: Message has no value.");
                }
            }
        };

        let mut args  = FluentArgs::new();
        for kv in args_pairs {
            args.set(kv.0, 
                kv.1);
        }

        let mut errors = vec![];
        let value = bs.bundle.format_pattern(pattern, Some(&args), &mut errors);
        value.to_string()
    } else {
        panic!("MFH: Language bundle mutex poisoned")
    }
}


///
/// run before use any build_language_* functions
/// if desired_lang is None, use the sys_locale::get_locale function.
/// if lang_dir is None, use the dir [`i18n/fluent`]
///
pub fn init_lang(desired_lang: Option<String>, lang_dir: Option<String>) {
    init_lang_with_donnot_panic(desired_lang, lang_dir, true);
}

pub fn init_lang_with_donnot_panic(desired_lang: Option<String>, lang_dir: Option<String>, donnot_panic: bool) {
    if LANG
        .set(Mutex::new(
                Arc::new(
                    LanguageSystem::new(desired_lang, lang_dir, donnot_panic))))
            .is_err()  {
                panic!("MFH: set initialized Language system failed.");
    }

}




#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        init_lang(None, None, );
        {
            let dest = format!("{}", build_language_0("a"));
            assert_eq!(dest, "<NoSuchMessage - a>");
        }
        {
            let dest = format!("{}", build_language_0("abc 啊"));
            assert_eq!(dest, "<NoSuchMessage - abc 啊>");
        }
        {
            let dest = format!("{}", build_language_1("abc 啊", "cde", 3.));
            assert_eq!(dest, "<NoSuchMessage - [<abc 啊>,cde]>");
        }
    }
}
