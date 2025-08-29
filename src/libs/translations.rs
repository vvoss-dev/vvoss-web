use std::collections::HashMap;

#[derive(Clone)]
pub struct Translations {
    pub strings: HashMap<String, HashMap<String, String>>,
}

impl Translations {
    /// Load translations from CSV file
    pub fn from_csv(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let csv_content = std::fs::read_to_string(path)?;
        let mut strings: HashMap<String, HashMap<String, String>> = HashMap::new();
        
        for (i, line) in csv_content.lines().enumerate() {
            if i == 0 { continue; } // Skip header
            
            let parts: Vec<&str> = line.split(';').collect();
            if parts.len() == 3 {
                let key = parts[0];
                let text = parts[1];
                let locale = parts[2];
                
                strings.entry(locale.to_string())
                    .or_insert_with(HashMap::new)
                    .insert(key.to_string(), text.to_string());
            }
        }
        
        Ok(Translations { strings })
    }
    
    /// Get translation for a key in a specific locale
    pub fn get(&self, locale: &str, key: &str) -> Option<&str> {
        self.strings.get(locale)
            .and_then(|locale_strings| locale_strings.get(key))
            .map(|s| s.as_str())
    }
    
    /// Get all translations for a locale with fallback
    pub fn get_locale_with_fallback(&self, locale: &str, fallback: &str) -> HashMap<String, String> {
        self.strings.get(locale)
            .or_else(|| self.strings.get(fallback))
            .cloned()
            .unwrap_or_default()
    }
}