use rstest::*;

use super::*;

mod url_parse {
    use super::*;

    #[rstest]
    fn test_parse_normal() {
        let url = Url::parse("http://localhost").unwrap();
        let url = url
            .join("/?canisterId=r7inp-6aaaa-aaaaa-aaabq-cai")
            .unwrap();
        assert_eq!(url.path(), "/");
        assert_eq!(url.query_pairs().count(), 1);
        assert_eq!(
            url.query_pairs()
                .find(|(k, _)| k == "canisterId")
                .unwrap()
                .1,
            "r7inp-6aaaa-aaaaa-aaabq-cai"
        );
    }

    #[rstest]
    fn test_parse_multiple_query() {
        let url = Url::parse("http://localhost").unwrap();
        let url = url
            .join("/?canisterId=r7inp-6aaaa-aaaaa-aaabq-cai&name=nice")
            .unwrap();
        assert_eq!(url.path(), "/");
        assert_eq!(url.query_pairs().count(), 2);
        assert_eq!(
            url.query_pairs()
                .find(|(k, _)| k == "canisterId")
                .unwrap()
                .1,
            "r7inp-6aaaa-aaaaa-aaabq-cai"
        );
        assert_eq!(
            url.query_pairs().find(|(k, _)| k == "name").unwrap().1,
            "nice"
        );
    }

    #[rstest]
    fn test_parse_multiple_value_w_the_same_name() {
        let url = Url::parse("http://localhost").unwrap();
        let url = url
            .join("/?canisterId=r7inp-6aaaa-aaaaa-aaabq-cai&canisterId=nice")
            .unwrap();
        assert_eq!(url.path(), "/");
        assert_eq!(url.query_pairs().count(), 2);
        let values: Vec<String> = url
            .query_pairs()
            .filter(|(k, _)| k == "canisterId")
            .map(|(_, v)| v.to_string())
            .collect();
        assert_eq!(values.len(), 2);
        assert_eq!(values[0], "r7inp-6aaaa-aaaaa-aaabq-cai");
        assert_eq!(values[1], "nice");
    }
}
