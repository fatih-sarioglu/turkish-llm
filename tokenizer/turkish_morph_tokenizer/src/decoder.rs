use std::collections::HashMap;

pub struct TurkishDecoder {
    reverse_dict: HashMap<i32, Vec<String>>,
}

// Constants for vowel/consonant sets
const ALL_VOWELS: &str = "aeıioöuüâ";
const INCE_VOWELS: &str = "eiöü";
const AI_VOWELS: &str = "aıâ";
const EI_VOWELS: &str = "ei";
const OU_VOWELS: &str = "ou";
const HARD_CONSONANTS: &str = "fstkçşhp";
const WHITESPACE: &str = " \n\t";

impl TurkishDecoder {
    pub fn new(reverse_dict: HashMap<i32, Vec<String>>) -> Self {
        Self { reverse_dict }
    }

    // modification: added this helper function to handle Turkish-specific lowercase conversion
    fn turkish_to_lowercase(c: char) -> char {
        match c {
            'İ' => 'i',
            'I' => 'ı',
            _ => c.to_lowercase().next().unwrap_or(c),
        }
    }

    // modification: added this helper function to check for uppercase letters in a string, considering Turkish-specific cases
    fn is_uppercase_turkish_letter(c: char) -> bool {
        matches!(c,
            'A'|'B'|'C'|'Ç'|'D'|'E'|'F'|'G'|'Ğ'|'H'|'I'|'İ'|
            'J'|'K'|'L'|'M'|'N'|'O'|'Ö'|'P'|'R'|'S'|'Ş'|'T'|
            'U'|'Ü'|'V'|'Y'|'Z'
        )
    }

    // modification: added this helper function to check for vowels in a string, considering Turkish-specific cases
    fn has_vowel(s: &str) -> bool {
        s.chars().any(|c| ALL_VOWELS.contains(Self::turkish_to_lowercase(c)))
    }

    // modification: added this helper function to check if a word starts with a vowel, considering Turkish-specific cases
    fn starts_with_vowel(&self, word: &str) -> bool {
        word.chars().next().map_or(false, |c| ALL_VOWELS.contains(Self::turkish_to_lowercase(c)))
    }

    // modification: added this helper function to check if a word ends with a vowel, considering Turkish-specific cases
    fn ends_with_vowel(&self, word: &str) -> bool {
        word.chars().last().map_or(false, |c| ALL_VOWELS.contains(Self::turkish_to_lowercase(c)))
    }

    // modification: added this helper function to check if a word ends with any character from a given set, considering Turkish-specific cases
    fn ends_with_any(&self, word: &str, charset: &str) -> bool {
        for c in word.chars().rev() {
            let lc = Self::turkish_to_lowercase(c);
            if charset.contains(lc) { return true; }
            if ALL_VOWELS.contains(lc) { return false; }
        }
        false
    }
    
    fn ends_with_ince(&self, word: &str) -> bool {
        match word {
            // modification: added more words
            " saat" | " kilovatsaat" | " ziraat" | " itaat" | " istikbal" | " festival" | " sembol" | " ideal" | " ihtimal" | " hayal" | " dikkat" | " rol" => true,
            _ => self.ends_with_any(word, INCE_VOWELS),
        }
    }

    fn ends_with_sert_unsuz(&self, word: &str) -> bool {
        word.chars().last().map_or(false, |c| HARD_CONSONANTS.contains(c))
    }
    
    fn get_vowel_suffix_index(&self, prev_token: &str) -> usize {
        // Loanword exceptions: final-a words whose internal vowels are front (take front harmony)
        if prev_token == " dikkat" { return 1; }
        // No vowel context: default to back unrounded (ı) rather than front rounded (ü)
        if !Self::has_vowel(prev_token) { return 0; }
        if self.ends_with_any(prev_token, AI_VOWELS) {
            0
        } else if self.ends_with_any(prev_token, EI_VOWELS) {
            1
        } else if self.ends_with_any(prev_token, OU_VOWELS) {
            2
        } else {
            3
        }
    }

    fn handle_la_le_suffix(&self, prev_token: &str, suffixes: &[String], end_of_word: bool) -> String {
        if self.ends_with_vowel(prev_token) && end_of_word {
            if self.ends_with_ince(prev_token) {
                suffixes[3].clone() // yle
            } else {
                suffixes[2].clone() // yla
            }
        } else {
            if self.ends_with_ince(prev_token) {
                suffixes[1].clone() // le
            } else {
                suffixes[0].clone() // la
            }
        }
    }

    fn handle_da_de_suffix(&self, prev_token: &str, suffixes: &[String]) -> String {
        if self.ends_with_sert_unsuz(prev_token) {
            if self.ends_with_ince(prev_token) {
                suffixes[3].clone() // te
            } else {
                suffixes[2].clone() // ta
            }
        } else {
            if self.ends_with_ince(prev_token) {
                suffixes[1].clone() // de
            } else {
                suffixes[0].clone() // da
            }
        }
    }

    fn handle_di_du_suffix(&self, prev_token: &str, suffixes: &[String]) -> String {
        let base_index = self.get_vowel_suffix_index(prev_token);
        if self.ends_with_sert_unsuz(prev_token) {
            suffixes[base_index + 4].clone()
        } else {
            suffixes[base_index].clone()
        }
    }
    
    fn handle_lik_suffix(&self, i: usize, ids: &[i32], prev_token: &str, suffixes: &[String]) -> String {
        // modification:
        // when lık/lik was the last token, it didn't handle the harmony
        let base_index = self.get_vowel_suffix_index(prev_token);
        if i >= ids.len() - 1 {
            return suffixes[base_index].clone();
        }
        
        let next_token = &self.reverse_dict[&ids[i + 1]][0];
        // let base_index = self.get_vowel_suffix_index(prev_token);
        
        if self.starts_with_vowel(next_token) {
            suffixes[base_index + 4].clone()
        } else {
            suffixes[base_index].clone()
        }
    }

    fn handle_cik_suffix(&self, i: usize, ids: &[i32], prev_token: &str, suffixes: &[String]) -> String {
        if i >= ids.len() - 1 {
            return suffixes[0].clone();
        }
        
        let next_token = &self.reverse_dict[&ids[i + 1]][0];
        let base_index = self.get_vowel_suffix_index(prev_token);
        
        let offset = if self.starts_with_vowel(next_token) {
            if self.ends_with_sert_unsuz(prev_token) { 12 } else { 8 }
        } else {
            if self.ends_with_sert_unsuz(prev_token) { 4 } else { 0 }
        };
        
        suffixes[base_index + offset].clone()
    }
    
    fn handle_mak_suffix(&self, i: usize, ids: &[i32], prev_token: &str, suffixes: &[String]) -> String {
        // modification:
        // when mak/mek was the last token, it didn't handle the harmony
        let base_index = if self.ends_with_ince(prev_token) { 1 } else { 0 };
        if i >= ids.len() - 1 {
            return suffixes[base_index].clone();
        }
        
        let next_token = &self.reverse_dict[&ids[i + 1]][0];
        // let base_index = if self.ends_with_ince(prev_token) { 1 } else { 0 };
        
        if self.starts_with_vowel(next_token) {
            suffixes[base_index + 2].clone()
        } else {
            suffixes[base_index].clone()
        }
    }
    
    fn handle_acak_suffix(&self, i: usize, ids: &[i32], prev_token: &str, suffixes: &[String]) -> String {
        let is_vowel_ending = self.ends_with_vowel(prev_token);
        let is_ince = self.ends_with_ince(prev_token);
        
        let is_vowel_starting = if i < ids.len() - 1 {
             let next_token = &self.reverse_dict[&ids[i + 1]][0];
             self.starts_with_vowel(next_token)
        } else {
             false
        };
        
        if is_vowel_starting {
            if is_vowel_ending {
                suffixes[if is_ince { 7 } else { 6 }].clone()
            } else {
                 suffixes[if is_ince { 3 } else { 2 }].clone()
            }
        } else {
            if is_vowel_ending {
                 suffixes[if is_ince { 5 } else { 4 }].clone()
            } else {
                 suffixes[if is_ince { 1 } else { 0 }].clone()
            }
        }
    }

    // modification: added this helper function to determine the correct suffix variant for number-based suffixes based on the last two digits of the number
    fn number_spoken_ending(n: u64) -> &'static str {
        if n == 0 { return "ır"; }            // sıfır

        // ones digit wins if non-zero
        let ones = n % 10;
        if ones != 0 {
            return match ones {
                1 => "ir",   // bir   – soft r,  front
                2 => "iki",  // iki   – vowel,   front
                3 => "üç",   // üç    – hard ç,  front
                4 => "ört",  // dört  – hard t,  front
                5 => "eş",   // beş   – hard ş,  front
                6 => "ı",    // altı  – vowel,   back
                7 => "edi",  // yedi  – vowel,   front
                8 => "iz",   // sekiz – soft z,  front
                9 => "uz",   // dokuz – soft z,  back
                _ => unreachable!(),
            };
        }

        // tens digit wins if non-zero (ones already zero)
        let tens = (n % 100) / 10;
        if tens != 0 {
            return match tens {
                1 => "on",   // on       – soft n,  back
                2 => "mi",   // yirmi    – vowel,   front
                3 => "uz",   // otuz     – soft z,  back
                4 => "ırk",  // kırk     – hard k,  back
                5 => "li",   // elli     – vowel,   front
                6 => "ış",   // altmış   – hard ş,  back
                7 => "iş",   // yetmiş   – hard ş,  front
                8 => "en",   // seksen   – soft n,  front
                9 => "an",   // doksan   – soft n,  back
                _ => unreachable!(),
            };
        }

        // hundreds: always "yüz"
        if (n % 1_000) / 100 != 0 { return "üz"; }

        // thousands: always "bin"
        if (n % 1_000_000) / 1_000 != 0 { return "in"; }

        // millions: "milyon"
        if (n % 1_000_000_000) / 1_000_000 != 0 { return "on"; }

        // billions: "milyar"
        "ar"
    }


    // modification: added this helper function to determine the vowel harmony class of a letter
    // all consonant letter-names end in a vowel in Turkish (be, ce, de, …),
    // so this also serves as a proxy for the final consonant's harmony class.
    fn letter_name_vowel(c: char) -> Option<&'static str> {
        match c {
            'A'       => Some("a"),   // a
            'B'       => Some("e"),   // be
            'C'       => Some("e"),   // ce
            'Ç'       => Some("e"),   // çe
            'D'       => Some("e"),   // de
            'E'       => Some("e"),   // e
            'F'       => Some("e"),   // fe
            'G'       => Some("e"),   // ge
            'Ğ'       => Some("e"),   // yumuşak ge
            'H'       => Some("e"),   // he
            'I'       => Some("ı"),   // ı  (undotted)
            'İ'       => Some("i"),   // i  (dotted, U+0130)
            'J'       => Some("e"),   // je
            'K'       => Some("a"),   // ka
            'L'       => Some("e"),   // le
            'M'       => Some("e"),   // me
            'N'       => Some("e"),   // ne
            'O'       => Some("o"),   // o
            'Ö'       => Some("ö"),   // ö
            'P'       => Some("e"),   // pe
            'R'       => Some("e"),   // re
            'S'       => Some("e"),   // se
            'Ş'       => Some("e"),   // şe
            'T'       => Some("e"),   // te
            'U'       => Some("u"),   // u
            'Ü'       => Some("ü"),   // ü
            'V'       => Some("e"),   // ve
            'Y'       => Some("e"),   // ye
            'Z'       => Some("e"),   // ze
            _         => None,
        }
    }


    /// given the text token that immediately precedes an apostrophe, returns
    /// a synthetic phonological-tail string suitable as a `prev_token` argument
    /// for all the suffix-selection helpers.  Returns `None` for ordinary words
    /// (caller falls back to normal vowel scanning).
    fn vowel_context_for_apostrophe_stem(stem: &str) -> Option<String> {
        let trimmed = stem.trim();

        // ── numbers: pure digit string ────────────────────────────────────────
        if !trimmed.is_empty() && trimmed.chars().all(|c| c.is_ascii_digit()) {
            if let Ok(n) = trimmed.parse::<u64>() {
                return Some(Self::number_spoken_ending(n).to_string());
            }
        }

        // ── acronyms: all uppercase Turkish letters, optional trailing '.' ───
        let base = trimmed.trim_end_matches('.');
        if !base.is_empty() && base.chars().all(Self::is_uppercase_turkish_letter) {
            // Use the last letter's Turkish name as the phonological tail.
            if let Some(last) = base.chars().last() {
                if let Some(tail) = Self::letter_name_vowel(last) {
                    return Some(tail.to_string());
                }
            }
        }

        None
    }

    
    fn select_correct_suffix(&self, i: usize, ids: &[i32], prev_token: &str) -> String {
        let token_id = ids[i];
        let suffixes = &self.reverse_dict[&token_id];
        
        // modification
        // invariant suffixes: never apply vowel harmony
        if token_id == 20007 || token_id == 20053 { // 20007: -ken, 20053: -gil
            return suffixes[0].clone();
        }


        if token_id < 20013 {
             if self.ends_with_ince(prev_token) { suffixes[1].clone() } else { suffixes[0].clone() }
        } else if token_id < 20023 {
             suffixes[self.get_vowel_suffix_index(prev_token)].clone()
        } else if token_id == 20023 { // la, le
             let mut end_of_word = true;
             if i < ids.len() - 1 {
                 let _next_token = &self.reverse_dict[&ids[i + 1]][0];
                 if !WHITESPACE.contains(_next_token.chars().next().unwrap_or(' ')) {
                     end_of_word = false;
                 }
             }
             self.handle_la_le_suffix(prev_token, suffixes, end_of_word)
        } else if token_id <= 20025 { // da, de, tan...
             self.handle_da_de_suffix(prev_token, suffixes)
        } else if token_id < 20029 { // di, du...
             self.handle_di_du_suffix(prev_token, suffixes)
        } else if token_id == 20029 { // lik
             self.handle_lik_suffix(i, ids, prev_token, suffixes)
        } else if token_id == 20030 { // cik
             self.handle_cik_suffix(i, ids, prev_token, suffixes)
        } else if token_id == 20031 { // mak
             self.handle_mak_suffix(i, ids, prev_token, suffixes)
        } else if token_id == 20032 { // acak
             self.handle_acak_suffix(i, ids, prev_token, suffixes)
        } else {
             suffixes[0].clone()
        }
    }

    fn select_correct_root(&self, i: usize, ids: &[i32]) -> String {
        let token_id = ids[i];
        let tokens = &self.reverse_dict[&token_id];
        
        // === EXCEPTIONS: Roots that should NOT soften ===
        // 204 (hayat), 220 (belirt), 298 (meslek)
        if token_id == 204 || token_id == 220 || token_id == 298 {
             return tokens[0].clone();
        }

        // Special case: üçlü (2227) - always return üçlü (variant 1) unless specific context
        if token_id == 2227 {
             if tokens.len() > 1 { return tokens[1].clone(); } else { return tokens[0].clone(); }
        }

        // Akış (aka/akı) Exception (2199) - Default to "akı" (variant 1) 
        if token_id == 2199 {
            if i < ids.len() - 1 {
                 let next_id = ids[i+1];
                 let next_str = &self.reverse_dict[&next_id][0];
                 // Use "aka" only when followed by vowel-starting suffixes like "acak"
                 if next_str.starts_with('a') || next_str.starts_with('e') {
                      return tokens[0].clone(); // "aka" for "akacak"
                 }
            }
            // Default to "akı"
            if tokens.len() > 1 { return tokens[1].clone(); } else { return tokens[0].clone(); }
        }

        // Ata/Atı Exception (2212) - for "atılırsa", "atılmak", "atıyorlar", "atık" etc.
        // Use "atı" (variant 1) when followed by 'l', 'y', or 'k' (atık=waste is more common than atak=bold)
        if token_id == 2212 {
            if tokens.len() > 1 && i < ids.len() - 1 {
                 let next_str = &self.reverse_dict[&ids[i+1]][0];
                 let ns = next_str.trim();
                 if ns.starts_with('l') || ns.starts_with('y') || ns.starts_with('k') {
                      return tokens[1].clone(); // "atı" + "lırsa/yacak/k" = "atılırsa/atıyacak/atık"
                 }
            }
            return tokens[0].clone(); // "ata" by default
        }
        
        // Yaşına (yaşa/yaşı) Exception (2209)
        if token_id == 2209 {
             if i < ids.len() - 1 {
                 // 20188 = 'na'
                 if ids[i+1] == 20188 {
                      if tokens.len() > 1 { return tokens[1].clone(); } else { return tokens[0].clone(); }
                 }
                // modification: narrow for yor (yaşıyor)
                if ids[i+1] == 20041 {
                    // return narrowed form: yaşa -> yaşı
                    let original = &tokens[0];
                    let mut s = original.trim_start().to_string();
                    s.pop(); s.push('ı');
                    return if original.starts_with(' ') { format!(" {}", s) } else { s };
                }
             }
             return tokens[0].clone();
        }
        
        // Alın (alın/aln) Exception (182) - Default to "alın" (variant 0)
        if token_id == 182 {
             if i < ids.len() - 1 {
                 let next_id = ids[i+1];
                 // Only drop vowel for simple possessive suffixes (ı, i, u, ü)
                 if next_id == 20034 || next_id == 20033 || next_id == 20035 || next_id == 20036 {
                      if tokens.len() > 1 { return tokens[1].clone(); } else { return tokens[0].clone(); }
                 }
             }
             return tokens[0].clone();
        }

        // Ilim/Ilm Exception (166) - Default to "ilim" (variant 0)
        if token_id == 166 {
            if tokens.len() > 1 && i < ids.len() - 1 {
                let next_id = ids[i+1];
                // Only use "ilm" for possessive/buffer case (ilmi, ilme) id 20033 ('i'), 20038 ('e')
                if next_id == 20033 || next_id == 20038 {
                     return tokens[1].clone(); // "ilm" + i = "ilmi"
                }
            }
            return tokens[0].clone(); // Default to "ilim"
        }

        // Boya/Boyu Exception (2220) - "boya" (paint) vs "boyu" (height)
        // Use "boyu" (variant 1) by default
        if token_id == 2220 {
            if tokens.len() > 1 && i < ids.len() - 1 {
                let next_id = ids[i+1];
                let next_str = &self.reverse_dict[&next_id][0];
                // Use "boya" only when followed by actual suffix tokens starting with 'n', 'm', 'l', 'd'
                if next_id >= 20000 && !next_str.trim().is_empty() {
                    let first_char = next_str.trim().chars().next().unwrap();
                    if "nmld".contains(first_char) {
                        return tokens[0].clone(); // "boya"
                    }
                }
            }
            if tokens.len() > 1 { return tokens[1].clone(); } else { return tokens[0].clone(); } // "boyu" by default
        }

        // Bile/Bili Exception (2307) - for "bilir", "biliyor" vs "biler"
        if token_id == 2307 {
            if tokens.len() > 1 && i < ids.len() - 1 {
                let next_str = &self.reverse_dict[&ids[i+1]][0];
                if next_str.trim().starts_with('r') || next_str.trim() == "yor" {
                    return tokens[1].clone(); // "bili" + "r" = "bilir"
                }
            }
            return tokens[0].clone(); // Default to "bile"
        }
        
        // Ada/Adı Exception (2218) - Default to "adı" (variant 1)
        if token_id == 2218 {
            if i < ids.len() - 1 {
                 let next_id = ids[i+1];
                 // Use "ada" when followed by specific suffix IDs for island/place-name paradigm
                 // 20017 = 'yı', 32725 = BPE yı, 20002 = 'ma/me', 32763 = BPE ma
                 // Also: uppercase + root + 'na' (20188) = "Adana" proper noun
                 let is_adana = i > 0 && ids[i-1] == 0 && next_id == 20188;
                 if next_id == 20017 || next_id == 32725 || next_id == 20002 || next_id == 32763 || is_adana {
                      return tokens[0].clone(); // "ada"
                 }
            }
            // Default to "adı" for most cases (adını, adından, adıyla...)
            if tokens.len() > 1 { return tokens[1].clone(); } else { return tokens[0].clone(); }
        }

        // Kap/Kab Exception (336) - favor "kapı" (door) over "kab" (container) context
        if token_id == 336 {
            if tokens.len() > 1 && i < ids.len() - 1 {
                let next_str = &self.reverse_dict[&ids[i+1]][0];
                // If followed by vowel (which causes softening default), check if it looks like possessive plural
                if next_str.trim().starts_with(|c: char| "aeıioöuüAEIİOÖUÜ".contains(c)) {
                    return tokens[0].clone(); // Keep "kap"
                }
            }
            return tokens[0].clone(); // Default "kap"
        }
        
        // Emekli/Emekle Exception (2295) - Default to "emekli" (variant 1)
        if token_id == 2295 {
            if i < ids.len() - 1 {
                 let next_id = ids[i+1];
                 // 20041 = 'yor' - for "emekliyor" use base form
                 if next_id == 20041 {
                      return tokens[0].clone(); // "emekle" + yor = emekliyor
                 }
            }
            // Default to "emekli" 
            if tokens.len() > 1 { return tokens[1].clone(); } else { return tokens[0].clone(); }
        }
        
        // Tutuk/Tutuğ/Tutk Exception (107) - for "tutkun"
        if token_id == 107 {
            if tokens.len() > 2 && i < ids.len() - 1 {
                 let next_str = &self.reverse_dict[&ids[i+1]][0];
                 // Check if next token starts with 'u' (un, unlar, etc.)
                 if next_str.trim().starts_with('u') {
                      return tokens[2].clone(); // "tutk" + "un" = "tutkun"
                 }
            }
            return tokens[0].clone();
        }
        
        // Başla/Başlı Exception (2206) - for "başlıca"
        if token_id == 2206 {
            if tokens.len() > 1 && i < ids.len() - 1 {
                 let next_id = ids[i+1];
                 // 20005 = 'ça/çe' suffix, 20047 = 'ce', 20207 = BPE 'ca'
                 if next_id == 20005 || next_id == 20047 || next_id == 20207 {
                      return tokens[1].clone(); // "başlı" + "ca" = "başlıca"
                 }
            }
            // Continue to existing logic below
        }
        
        // Dip/Dib Exception (2406) - soften to "dib" before vowel suffixes
        if token_id == 2406 {
            if tokens.len() > 1 && i < ids.len() - 1 {
                let next_str = &self.reverse_dict[&ids[i+1]][0];
                if next_str.trim().starts_with(|c: char| "aeıioöuüAEIİOÖUÜ".contains(c)) {
                    return tokens[1].clone(); // "dib" + "inde" = "dibinde"
                }
            }
            return tokens[0].clone(); // "dip" by default
        }
        
        // de (19531) / ye (19968) / başla (2206) narrowing logic
        if token_id == 19531 || token_id == 19968 || token_id == 2206 {
             let mut should_narrow = false;
             
             if i < ids.len() - 1 {
                 let next_token = &self.reverse_dict[&ids[i + 1]][0];
                 // Check for "yor" string match (covers 32621, 20041 etc)
                 if next_token.contains("yor") {
                     should_narrow = true;
                 } else if let Some(suff_forms) = self.reverse_dict.get(&ids[i+1]) {
                     if suff_forms.iter().any(|s| s.starts_with(|c| ALL_VOWELS.contains(c))) {
                          // Only for de/ye, not başla (start vowel usually narrows de/ye->di/yi but başla->başlı?)
                          // Actually 2206 (başla/başlı) only narrows for YOR usually. 
                          // "Başla" + "acak" -> "Başlayacak" (no narrowing)
                          // "Başla" + "yıp" -> "Başlayıp"
                          // So for 2206, ONLY narrow if "yor"
                          if token_id != 2206 {
                              should_narrow = true;
                          }
                     }
                 }
             }
             
             if should_narrow {
                 // For 2206: başla -> başlı (variant 1)
                 if token_id == 2206 {
                      return tokens[1].clone();
                 }
                 
                 let original = &tokens[0];
                 if original.ends_with('e') {
                     let mut s = original.clone();
                     s.pop();
                     s.push('i');
                     return s;
                 } else if original.ends_with('E') {
                     let mut s = original.clone();
                     s.pop();
                     s.push('İ');
                     return s;
                 }
             }
             return tokens[0].clone();
        }
        
        // Range 100-2080: Generic Softening
        if token_id >= 100 && token_id < 2080 {
             // Skip if NO_SOFTENING_ROOTS (already handled) or EXCEPTION_ROOTS
             
             if i < ids.len() - 1 {
                 let next_token = &self.reverse_dict[&ids[i + 1]][0];
                 if self.starts_with_vowel(next_token) {
                     if tokens.len() > 1 { return tokens[1].clone(); }
                 } else if token_id <= 110 && next_token.trim() == "ı" {
                     if tokens.len() > 2 { return tokens[2].clone(); }
                 }
             }
             return tokens[0].clone();
        }

        // Special roots where tokens[1] is the correct default (not only before -yor):
        // 2298 türe/türü: standalone noun türü (its kind/type) is far more common than verb türe
        // 2234 ikile/ikili: adjective ikili (dual/binary) is far more common than verb stem ikile
        if token_id == 2298 || token_id == 2234 {
             if tokens.len() > 1 { return tokens[1].clone(); } else { return tokens[0].clone(); }
        }

        // Root 2303: dene/deni — passive stem 'deni' before passive 'l', aorist 'r', or yor
        // dene (try) takes 'deni' in passive/aorist: denilen, denir, denilebilir
        // but denendi = dene+ndi is correct with 'dene'
        if token_id == 2303 {
            if tokens.len() > 1 && i < ids.len() - 1 {
                let next_str = &self.reverse_dict[&ids[i + 1]][0];
                let ns = next_str.trim();
                if ns.starts_with('l') || ns == "r" || ns.contains("yor") {
                    return tokens[1].clone(); // deni
                }
            }
            return tokens[0].clone(); // dene
        }

        // Root 2300: ele/eli — noun form 'eli' (hand) before n-initial possessive/locative or yor
        // ele (criticize/select) vs eli (hand): eli+nde=elinde, eli+ni=elini, ele+niyor=eleniyor
        if token_id == 2300 {
            if tokens.len() > 1 && i < ids.len() - 1 {
                let next_str = &self.reverse_dict[&ids[i + 1]][0];
                let ns = next_str.trim();
                if ns.starts_with('n') || ns.contains("yor") {
                    return tokens[1].clone(); // eli
                }
            }
            return tokens[0].clone(); // ele
        }

        // Range 2080-2315: Narrowing (e.g. verbs like demek/yemek other than de/ye)
        if token_id >= 2080 && token_id < 2315 {
             if i < ids.len() - 1 {
                 let next_token = &self.reverse_dict[&ids[i + 1]][0];
                 if next_token.contains("yor") {
                     if tokens.len() > 1 { return tokens[1].clone(); }
                 }
                 // Python Check: else return variant 0
             }
             return tokens[0].clone();
        }
        
        tokens[0].clone()
    }
    // Capitalize token with proper Turkish I handling
    fn capitalize_token(token: &str) -> String {
        if token.starts_with(' ') {
             // Preserve leading space
             let mut chars = token.chars();
             let _first = chars.next().unwrap(); // ' '
             
             // Find first non-space
             let rest = chars.as_str();
             if rest.is_empty() { return token.to_string(); }
             
             let mut rest_chars = rest.chars();
             if let Some(c) = rest_chars.next() {
                 let cap = match c {
                     'i' => "İ".to_string(),
                     'ı' => "I".to_string(),
                     _ => c.to_uppercase().to_string(),
                 };
                 format!(" {}{}", cap, rest_chars.as_str())
             } else {
                 token.to_string()
             }
        } else {
             let mut chars = token.chars();
             if let Some(c) = chars.next() {
                 let cap = match c {
                     'i' => "İ".to_string(),
                     'ı' => "I".to_string(),
                     _ => c.to_uppercase().to_string(),
                 };
                 format!("{}{}", cap, chars.as_str())
             } else {
                 String::new()
             }
        }
    }

    pub fn decode(&self, ids: Vec<i32>) -> String {
        if ids.is_empty() { return String::new(); }
        
        let mut text_parts: Vec<String> = Vec::with_capacity(ids.len());
        let mut i = 0;
        
        while i < ids.len() {
            let token_id = ids[i];
            
            if token_id == 0 && i < ids.len() - 1 { // uppercase
                // We must process the next token with full logic (softening/vowel drop)
                // before capitalizing it.
                // Determine if next is root or suffix to call correct method.
                let next_id = ids[i + 1];
                let resolved_token = if next_id < 20000 {
                     self.select_correct_root(i + 1, &ids)
                } else if next_id <= 20071 {
                     // Suffix context logic duplication or refactor?
                     // Python select_correct_root handles roots. 
                     // Only roots typically start a word/Sentence.
                     // But if Uppercase is applied to a suffix (unlikely but possible), 
                     // Python only calls select_correct_root in line 436.
                     self.select_correct_root(i + 1, &ids) 
                } else {
                     // BPE or other
                     if let Some(tokens) = self.reverse_dict.get(&next_id) {
                         tokens[0].clone()
                     } else {
                         String::new()
                     }
                };
                
                text_parts.push(Self::capitalize_token(&resolved_token));
                i += 2;
                continue;
            } else if token_id == 1 { // unknown
                text_parts.push("▁u▁".to_string());
            } else if let Some(tokens) = self.reverse_dict.get(&token_id) {
                if tokens.len() > 1 {
                    if token_id >= 20000 && token_id <= 20071 { // suffix
                         // Context construction (looking back up to 3 tokens)
                         let mut vowel_context_str = String::new();
                         let mut found_vowel = false;

                         // 1. Check immediate previous tokens for simple vowel presence
                         let mut j = (text_parts.len() as isize) - 1;
                         let mut tokens_checked = 0;
                         
                         while j >= 0 && tokens_checked < 3 {
                             let prev = &text_parts[j as usize];
                             
                             // modification for apostrophes: if we encounter an apostrophe, we should check the stem before it for vowel context, and skip the apostrophe itself
                             if !prev.trim().is_empty() {
                                // ── apostrophe boundary: check the stem that precedes it ──────
                                if prev == "'" || prev == "\u{2019}" {
                                    if j > 0 {
                                        let mut k = (j - 1) as usize;

                                        // skip a trailing period token (handles abbreviations like "Ş.")
                                        if text_parts[k].trim() == "." {
                                            if k > 0 { k -= 1; } else {
                                                tokens_checked += 1;
                                                j -= 1;
                                                continue;
                                            }
                                        }

                                        // ── 1. collect consecutive digit tokens → reconstruct full number ──
                                        // e.g. ["1","9","8","0"] → 1980
                                        let mut digit_chars: Vec<char> = Vec::new();
                                        let mut m = k as isize;
                                        while m >= 0 {
                                            let t = text_parts[m as usize].trim();
                                            if t.len() == 1 && t.chars().next().map_or(false, |c| c.is_ascii_digit()) {
                                                digit_chars.push(t.chars().next().unwrap());
                                                m -= 1;
                                            } else {
                                                break;
                                            }
                                        }
                                        if !digit_chars.is_empty() {
                                            digit_chars.reverse();
                                            let digit_str: String = digit_chars.into_iter().collect();
                                            if let Ok(n) = digit_str.parse::<u64>() {
                                                vowel_context_str = Self::number_spoken_ending(n).to_string();
                                                found_vowel = true;
                                                break;
                                            }
                                        }


                                        // ── 2. Collect consecutive single uppercase Turkish letter tokens ──
                                        // e.g. [" K","O","B","İ"] → last letter İ → "i"
                                        // After capitalize_token(), each letter is a 1-char trimmed string.
                                        let mut last_letter: Option<char> = None;
                                        let mut m = k as isize;
                                        while m >= 0 {
                                            let t = text_parts[m as usize].trim();
                                            let mut chars = t.chars();
                                            if let (Some(c), None) = (chars.next(), chars.next()) {
                                                if Self::is_uppercase_turkish_letter(c) {
                                                    if last_letter.is_none() { last_letter = Some(c); }
                                                    m -= 1;
                                                    continue;
                                                }
                                            }
                                            break;
                                        }
                                        if let Some(last) = last_letter {
                                            if let Some(tail) = Self::letter_name_vowel(last) {
                                                vowel_context_str = tail.to_string();
                                                found_vowel = true;
                                                break;
                                            }
                                        }


                                        // ── 3. Ordinary word before apostrophe ────────────────────────────
                                        // ordinary word before apostrophe (e.g. "Türkiye'nin")
                                        let stem = &text_parts[k];
                                        if Self::has_vowel(stem) {
                                            vowel_context_str = stem.clone();
                                            found_vowel = true;
                                            break;
                                        }
                                    }
                                    tokens_checked += 1;
                                    j -= 1;
                                    continue;
                                }
                                
                                // digit tokens: reconstruct full number and use spoken ending for harmony
                                {
                                    let t = prev.trim();
                                    if !t.is_empty() && t.chars().all(|c| c.is_ascii_digit()) {
                                        let mut digit_chars: Vec<char> = Vec::new();
                                        let mut m = j as isize;
                                        while m >= 0 {
                                            let tok = text_parts[m as usize].trim();
                                            if !tok.is_empty() && tok.chars().all(|c| c.is_ascii_digit()) {
                                                for c in tok.chars().rev() { digit_chars.push(c); }
                                                m -= 1;
                                            } else {
                                                break;
                                            }
                                        }
                                        digit_chars.reverse();
                                        let digit_str: String = digit_chars.into_iter().collect();
                                        if let Ok(n) = digit_str.parse::<u64>() {
                                            vowel_context_str = Self::number_spoken_ending(n).to_string();
                                            found_vowel = true;
                                            break;
                                        }
                                    }
                                }

                                // normal token: check if it has a vowel
                                // Found a non-empty token. Does it have a vowel?
                                if Self::has_vowel(prev) {
                                    vowel_context_str = prev.clone();
                                    found_vowel = true;
                                    break; // Found it!
                                }
                                tokens_checked += 1;
                             }
                             j -= 1;
                         }

                         // 2. If no vowel found in single tokens, look deeper by concatenating (depth 3)
                         if !found_vowel {
                             let mut depth = 0;
                             let mut temp_ctx = String::new();
                             let mut m = (text_parts.len() as isize) - 1;
                             
                             while m >= 0 && depth < 3 {
                                 let prev = &text_parts[m as usize];
                                 temp_ctx = prev.clone() + &temp_ctx; // Prepend
                                 if Self::has_vowel(&temp_ctx) {
                                     vowel_context_str = temp_ctx;
                                     break;
                                 }
                                 m -= 1;
                                 depth += 1;
                             }
                         }
                         
                         // Extend vowel_context with any immediately trailing non-vowel, non-digit token
                         // so ends_with_vowel reflects the actual surface boundary (e.g. nedenle+r+la).
                         if found_vowel && !text_parts.is_empty() {
                             let last = &text_parts[text_parts.len() - 1];
                             let t = last.trim();
                             if !Self::has_vowel(last) && !t.is_empty()
                                 && vowel_context_str != *last
                                 && !t.chars().all(|c| c.is_ascii_digit()) {
                                 vowel_context_str.push_str(t);
                             }
                         }

                         text_parts.push(self.select_correct_suffix(i, &ids, &vowel_context_str));
                    } else if token_id < 20000 { // root
                         text_parts.push(self.select_correct_root(i, &ids));
                    } else { // BPE (> 20071) -> Static
                         text_parts.push(tokens[0].clone());
                    }
                } else {
                    text_parts.push(tokens[0].clone());
                }
            } else {
                 text_parts.push("▁".to_string());
            }
            i += 1;
        }
        
        text_parts.join("")
    }
}
