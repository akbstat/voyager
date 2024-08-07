use regex::Regex;

#[derive(Debug, PartialEq, Eq)]
pub enum AnnotationKind {
    Main,
    Supp,
    Other,
}

pub fn annotation_kind(raw: &str) -> AnnotationKind {
    let re_other =
        Regex::new(r"^((If\s.+?then\s)|(Datepart\sof\s)|(Timepart\sof\s))?[A-Z0-9]{4,8}.*")
            .unwrap();

    if !re_other.is_match(&raw) {
        return AnnotationKind::Other;
    }

    // exclude domain declare
    let domain_re = Regex::new(r"^[A-Z]{2,6}\s?\(").unwrap();
    if domain_re.is_match(&raw) {
        return AnnotationKind::Other;
    }

    let re_supp = Regex::new(r"SUPP[A-Z]{2}").unwrap();

    if re_supp.is_match(&raw) {
        return AnnotationKind::Supp;
    }
    AnnotationKind::Main
}

#[cfg(test)]
mod tests {
    const MAIN_1: &str = "AESTDTC";
    const MAIN_2: &str = "LBTEST = Erythrocytes";
    const MAIN_3: &str = "VSORRES when VSTESTCD = TEMP";
    const MAIN_4: &str = "MISTAT = NOT DONE when MITESTCD = MIALL";
    const MAIN_5: &str = "TRORRES / TRORRESU when TRTESTCD = SUMDIAM";
    const MAIN_6: &str = "Datepart of ECSTDTC";
    const MAIN_7: &str = "Timepart of ECSTDTC";
    const SUPP_1: &str = "AESI in SUPPAE";
    const SUPP_2: &str = "PECLSIG=N in SUPPPE";
    const SUPP_3: &str = "If Normal then XONRIND = NORMAL in SUPPXO";
    const SUPP_4: &str = "DDORRES in SUPPDD when DDTESTCD = PRCDTH";
    const OTHER_1: &str = "[NOT SUBMITTED]";
    const OTHER_2: &str = "DM = 人口学特征";
    const OTHER_3: &str = "RELREC (Related Records)";
    const OTHER_4: &str = "See CRF Page";
    const OTHER_5: &str = "Note:";
    const OTHER_6: &str = "Linked to related AE record via RELREC";

    use super::*;
    #[test]
    fn annotation_kind_test() {
        assert_eq!(annotation_kind(MAIN_1), AnnotationKind::Main);
        assert_eq!(annotation_kind(MAIN_2), AnnotationKind::Main);
        assert_eq!(annotation_kind(MAIN_3), AnnotationKind::Main);
        assert_eq!(annotation_kind(MAIN_4), AnnotationKind::Main);
        assert_eq!(annotation_kind(MAIN_5), AnnotationKind::Main);
        assert_eq!(annotation_kind(MAIN_6), AnnotationKind::Main);
        assert_eq!(annotation_kind(MAIN_7), AnnotationKind::Main);
        assert_eq!(annotation_kind(SUPP_1), AnnotationKind::Supp);
        assert_eq!(annotation_kind(SUPP_2), AnnotationKind::Supp);
        assert_eq!(annotation_kind(SUPP_3), AnnotationKind::Supp);
        assert_eq!(annotation_kind(SUPP_4), AnnotationKind::Supp);
        assert_eq!(annotation_kind(OTHER_1), AnnotationKind::Other);
        assert_eq!(annotation_kind(OTHER_2), AnnotationKind::Other);
        assert_eq!(annotation_kind(OTHER_3), AnnotationKind::Other);
        assert_eq!(annotation_kind(OTHER_3), AnnotationKind::Other);
        assert_eq!(annotation_kind(OTHER_4), AnnotationKind::Other);
        assert_eq!(annotation_kind(OTHER_5), AnnotationKind::Other);
        assert_eq!(annotation_kind(OTHER_6), AnnotationKind::Other);
    }
}
