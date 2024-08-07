// use regex::Regex;

// pub struct ReCollection {
//     pub other: Regex,
//     pub supp: Regex,
//     pub if_then: Regex,
// }

// impl ReCollection {
//     pub fn new() -> ReCollection {
//         ReCollection {
//             other: Regex::new(
//                 r"^((If\s.+?then\s)|(Datepart\sof\s)|(Timepart\sof\s))?[A-Z0-9]{4,8}.*",
//             )
//             .unwrap(),
//             supp: Regex::new(r"SUPP[A-Z]{2}").unwrap(),
//             if_then: Regex::new(r"If\s\w+?\sthen\s(.*)").unwrap(),
//         }
//     }
// }
