pub struct TopicMatcher;

impl TopicMatcher {

    pub fn matches(

        subscription: &str,

        topic: &str,

    ) -> bool {

        if subscription == "*" {

            return true;
        }

        subscription == topic
    }
}