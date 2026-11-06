//! Recherche floue (PROMPT §1 et §10 « Recherche ») : tolère les accents, la casse,
//! la ponctuation, les fautes de frappe et les initiales (« gta »).
//! Aucune dépendance : tout est calculé ici, sur des titres déjà normalisés.

/// Met un titre sous une forme comparable : minuscules, sans accents,
/// ponctuation remplacée par des espaces, espaces uniques.
pub fn normalize(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut space = true;
    for c in text.chars().flat_map(char::to_lowercase) {
        let mapped: &str = match c {
            'à' | 'á' | 'â' | 'ã' | 'ä' | 'å' => "a",
            'ç' => "c",
            'è' | 'é' | 'ê' | 'ë' => "e",
            'ì' | 'í' | 'î' | 'ï' => "i",
            'ñ' => "n",
            'ò' | 'ó' | 'ô' | 'õ' | 'ö' | 'ø' => "o",
            'ù' | 'ú' | 'û' | 'ü' => "u",
            'ý' | 'ÿ' => "y",
            'œ' => "oe",
            'æ' => "ae",
            'ß' => "ss",
            // Marques déposées : on les retire sans couper le mot.
            '™' | '®' | '©' | '\'' | '’' => "",
            c if c.is_alphanumeric() => {
                out.push(c);
                space = false;
                continue;
            }
            _ => " ",
        };
        if mapped == " " {
            if !space {
                out.push(' ');
                space = true;
            }
        } else if !mapped.is_empty() {
            out.push_str(mapped);
            space = false;
        }
    }
    if out.ends_with(' ') {
        out.pop();
    }
    out
}

/// Note de pertinence d'un titre pour une recherche (les deux déjà normalisés).
/// `None` = ne correspond pas. Plus la note est haute, plus le jeu remonte.
pub fn score(query: &str, title: &str) -> Option<i32> {
    if query.is_empty() {
        return Some(0);
    }
    // 1. Le texte exact apparaît dans le titre.
    if let Some(pos) = title.find(query) {
        let at_word = pos == 0 || title.as_bytes().get(pos - 1) == Some(&b' ');
        let mut s = 1000 - pos.min(200) as i32;
        if at_word {
            s += 200;
        }
        if pos == 0 {
            s += 200;
        }
        return Some(s);
    }
    // 2. Chaque mot cherché trouve un mot du titre, avec quelques fautes permises.
    let words: Vec<&str> = title.split(' ').collect();
    let mut total = 0;
    let mut all = true;
    for q in query.split(' ') {
        match words.iter().filter_map(|w| word_score(q, w)).max() {
            Some(s) => total += s,
            None => {
                all = false;
                break;
            }
        }
    }
    if all {
        return Some(400 + total);
    }
    // 3. Initiales : « gta » → Grand Theft Auto, « rdr2 » → Red Dead Redemption 2.
    let compact: String = query.chars().filter(|c| *c != ' ').collect();
    if compact.chars().count() >= 2 && is_acronym(&compact, &words) {
        return Some(300);
    }
    None
}

/// Un mot cherché contre un mot du titre.
fn word_score(q: &str, w: &str) -> Option<i32> {
    if w.starts_with(q) {
        return Some(100);
    }
    if w.contains(q) {
        return Some(60);
    }
    let qn = q.chars().count();
    let allowed = match qn {
        0..=3 => 0,
        4..=7 => 1,
        _ => 2,
    };
    if allowed == 0 {
        return None;
    }
    // On compare au mot entier et au début du mot (frappe en cours).
    let prefix: String = w.chars().take(qn).collect();
    let d = distance(q, w).min(distance(q, &prefix));
    (d <= allowed).then(|| 50 - 15 * d as i32)
}

/// Distance d'édition avec inversion de deux lettres voisines (Damerau, version simple).
fn distance(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let (n, m) = (a.len(), b.len());
    let mut d = vec![vec![0usize; m + 1]; n + 1];
    for (i, row) in d.iter_mut().enumerate() {
        row[0] = i;
    }
    for (j, cell) in d[0].iter_mut().enumerate() {
        *cell = j;
    }
    for i in 1..=n {
        for j in 1..=m {
            let cost = usize::from(a[i - 1] != b[j - 1]);
            let mut v = (d[i - 1][j] + 1)
                .min(d[i][j - 1] + 1)
                .min(d[i - 1][j - 1] + cost);
            if i > 1 && j > 1 && a[i - 1] == b[j - 2] && a[i - 2] == b[j - 1] {
                v = v.min(d[i - 2][j - 2] + 1);
            }
            d[i][j] = v;
        }
    }
    d[n][m]
}

/// Les lettres de la recherche sont, dans l'ordre, les initiales des mots
/// (un chiffre peut aussi correspondre à un mot-nombre entier).
fn is_acronym(q: &str, words: &[&str]) -> bool {
    let mut it = words.iter().filter(|w| !w.is_empty());
    for c in q.chars() {
        loop {
            match it.next() {
                Some(w) if w.starts_with(c) => break,
                Some(_) => continue,
                None => return false,
            }
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hit(q: &str, title: &str) -> bool {
        score(&normalize(q), &normalize(title)).is_some()
    }

    #[test]
    fn normalizes_accents_marks_and_punctuation() {
        assert_eq!(normalize("Pokémon™: Écarlate"), "pokemon ecarlate");
        assert_eq!(normalize("  Baldur's   Gate 3 "), "baldurs gate 3");
        assert_eq!(normalize("Œuvre—finale"), "oeuvre finale");
    }

    #[test]
    fn finds_plain_and_accented_text() {
        assert!(hit("knight", "Hollow Knight"));
        assert!(hit("pokemon", "Pokémon Écarlate"));
        assert!(hit("ÉCARLATE", "Pokemon Ecarlate"));
        assert!(!hit("zelda", "Hades"));
    }

    #[test]
    fn tolerates_typos() {
        assert!(hit("holow knigt", "Hollow Knight"));
        assert!(hit("witcehr", "The Witcher 3: Wild Hunt"));
        assert!(hit("cyberpnuk", "Cyberpunk 2077"));
        // Trop court pour deviner : pas de faute permise sous 4 lettres.
        assert!(!hit("hdes", "Doom"));
        assert!(!hit("abc", "Hades"));
    }

    #[test]
    fn matches_initials() {
        assert!(hit("gta", "Grand Theft Auto V"));
        assert!(hit("rdr2", "Red Dead Redemption 2"));
        assert!(!hit("gtx", "Grand Theft Auto V"));
    }

    #[test]
    fn ranks_prefix_before_middle_before_typo() {
        let q = normalize("hades");
        let start = score(&q, &normalize("Hades II"));
        let middle = score(&q, &normalize("Return to Hades"));
        let typo = score(&normalize("hadse"), &normalize("Hades II"));
        assert!(start > middle);
        assert!(middle > typo);
        assert!(typo.is_some());
    }
}
