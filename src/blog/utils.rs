#[allow(dead_code)]
const STOP_WORDS: [&str; 177] = [
    "a",
    "about",
    "above",
    "after",
    "again",
    "against",
    "all",
    "am",
    "an",
    "and",
    "any",
    "are",
    "aren't",
    "as",
    "at",
    "be",
    "because",
    "been",
    "before",
    "being",
    "below",
    "between",
    "both",
    "but",
    "by",
    "can",
    "can't",
    "cannot",
    "could",
    "couldn't",
    "did",
    "didn't",
    "do",
    "does",
    "doesn't",
    "doing",
    "don't",
    "down",
    "during",
    "each",
    "few",
    "for",
    "from",
    "further",
    "had",
    "hadn't",
    "has",
    "hasn't",
    "have",
    "haven't",
    "having",
    "he",
    "he'd",
    "he'll",
    "he's",
    "her",
    "here",
    "here's",
    "hers",
    "herself",
    "him",
    "himself",
    "his",
    "how",
    "how's",
    "i",
    "i'd",
    "i'll",
    "i'm",
    "i've",
    "if",
    "in",
    "into",
    "is",
    "isn't",
    "it",
    "it's",
    "its",
    "itself",
    "let's",
    "me",
    "more",
    "most",
    "mustn't",
    "my",
    "myself",
    "no",
    "nor",
    "not",
    "now",
    "of",
    "off",
    "on",
    "once",
    "only",
    "or",
    "other",
    "ought",
    "our",
    "ours",
    "ourselves",
    "out",
    "over",
    "own",
    "same",
    "shan't",
    "she",
    "she'd",
    "she'll",
    "she's",
    "should",
    "shouldn't",
    "so",
    "some",
    "such",
    "than",
    "that",
    "that's",
    "the",
    "their",
    "theirs",
    "them",
    "themselves",
    "then",
    "there",
    "there's",
    "these",
    "they",
    "they'd",
    "they'll",
    "they're",
    "they've",
    "this",
    "those",
    "through",
    "to",
    "too",
    "under",
    "until",
    "up",
    "very",
    "was",
    "wasn't",
    "we",
    "we'd",
    "we'll",
    "we're",
    "we've",
    "were",
    "weren't",
    "what",
    "what's",
    "when",
    "when's",
    "where",
    "where's",
    "which",
    "while",
    "who",
    "who's",
    "whom",
    "why",
    "why's",
    "with",
    "won't",
    "would",
    "wouldn't",
    "you",
    "you'd",
    "you'll",
    "you're",
    "you've",
    "your",
    "yours",
    "yourself",
    "yourselves",
    "name",
];

#[allow(dead_code)]
pub fn remove_stopwords(input: &str) -> Vec<String> {
    // Split input into phrases (using whitespace or punctuation as delimiters)
    let phrases: Vec<&str> = input.split_whitespace().collect();

    // Filter out phrases containing any stop word
    let filtered_phrases: Vec<String> = phrases
        .into_iter()
        .filter(|phrase| {
            !STOP_WORDS.contains(phrase)
            // let words: Vec<String> = phrase.to_lowercase().split_whitespace().collect();
            // !words.iter().any(|word| stop_words.contains(word))
        })
        .map(|x| x.to_owned().to_lowercase())
        .collect();

    filtered_phrases
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::utils::doc::insert_tags;

    #[test]
    fn test_remove_stopwords() {
        let input = "Hello my name is David";
        let res = remove_stopwords(input);
        assert_eq!(res, Vec::from(["hello", "david"]));
    }

    #[test]
    fn test_query() {
        let res = insert_tags(
            "tags",
            "(doc_id, user_id, name, score)",
            vec![(1, 1, "Sankar", 1), (1, 1, "Boro", 1)],
        );
        assert_eq!(
            res,
            String::from(
                "INSERT INTO tags (doc_id, user_id, name, score) VALUES (1, 1, 'Sankar', 1), (1, 1, 'Boro', 1)"
            )
        );
    }
}
